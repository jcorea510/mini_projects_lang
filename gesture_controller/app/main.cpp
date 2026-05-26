#include <cstddef>
#include <iostream>
#include <filesystem>
#include <opencv2/core/mat.hpp>
#include <opencv2/dnn/dnn.hpp>
#include <opencv2/opencv.hpp>
#include <opencv2/videoio.hpp>
#include <print>
#include <string>
#include <vector>
#include <yolo_models.hpp>
#include <serialib.h>
#include <map>

namespace fs = std::filesystem;

static const std::vector<std::string> IMAGE_EXTENSIONS = {
    ".jpg", ".jpeg", ".png", ".bmp", ".tiff", ".webp"
};

std::optional<std::size_t> getUserInputInt(const std::string& line, std::size_t max) {
    std::size_t ret;

    while (!std::cin.eof()) {
        std::print("{} (0-{}): ", line, max);
        std::cin >> ret;

        if (!std::cin) {
            return {};
        }

        if (ret <= max) {
            return ret;
        }
    }
    return {};
}

bool is_image_file(const fs::path& p) {
    std::string ext = p.extension().string();
    std::transform(ext.begin(), ext.end(), ext.begin(), ::tolower);
    for (const auto& e : IMAGE_EXTENSIONS)
        if (ext == e) return true;
    return false;
}

std::vector<fs::path> collect_image_paths(const std::string& input) {
    std::vector<fs::path> paths;
    fs::path p(input);

    if (fs::is_regular_file(p) && is_image_file(p)) {
        paths.push_back(p);
    } else if (fs::is_directory(p)) {
        for (const auto& entry : fs::directory_iterator(p)) {
            if (entry.is_regular_file() && is_image_file(entry.path()))
                paths.push_back(entry.path());
        }
        std::sort(paths.begin(), paths.end());
    } else {
        std::cerr << "Error: '" << input << "' is not a valid image file or directory.\n";
    }
    return paths;
}

// Run inference and draw results on a single frame
std::optional<std::string> process_frame(cv::Mat& frame,
                   cv::dnn::Net& detection_net,
                   cv::dnn::Net& classification_net,
                   const std::vector<std::string>& class_list,
                   const std::vector<std::string>& classification_classes,
                   float cls_threshold = SCORE_THRESHOLD)
{
    cv::resize(frame, frame, cv::Size(640, 640), 0, 0, cv::INTER_LINEAR);

    std::vector<Detection> detections;
    detect(frame, detection_net, detections, class_list);

    std::optional<std::string> top_gesture;
    float top_conf = 0.f;

    for (const auto& det : detections) {
        cv::Rect safe_box = det.box & cv::Rect(0, 0, frame.cols, frame.rows);
        if (safe_box.area() == 0) continue;

        cv::Mat hand_crop = frame(safe_box).clone();
        Classification result;
        if (!classify(hand_crop, classification_net, result, classification_classes)) continue;
        if (result.confidence < cls_threshold) continue;

        std::string label = classification_classes[result.class_id]
                          + " (" + std::to_string(static_cast<int>(result.confidence * 100)) + "%)";
        cv::rectangle(frame, det.box, cv::Scalar(0, 255, 0), 2);
        int baseline = 0;
        cv::Size text_size = cv::getTextSize(label, cv::FONT_HERSHEY_SIMPLEX, 0.8, 2, &baseline);
        cv::Point labelin(det.box.x, std::max(det.box.y - 10, text_size.height + 5));
        cv::rectangle(frame,
                      labelin + cv::Point(0, baseline),
                      labelin + cv::Point(text_size.width, -text_size.height - 5),
                      cv::Scalar(0, 0, 0), cv::FILLED);
        cv::putText(frame, label, labelin,
                    cv::FONT_HERSHEY_SIMPLEX, 0.8, cv::Scalar(0, 255, 0), 2);

        if (result.confidence > top_conf) {
            top_conf    = result.confidence;
            top_gesture = classification_classes[result.class_id];
        }
    }
    return top_gesture;
}

void print_usage(const char* prog) {
    std::println("Usage:");
    std::println("  {} --camera                         Run on live webcam (default)", prog);
    std::println("  {} --images <file|dir>              Run on images", prog);
    std::println("  {} --threshold <0.0-1.0>            Classification confidence threshold", prog);
    std::println("  {} --serial                         Enable USB serial output", prog);
    std::println("  {} --serial-port <path>             Serial device (default: /dev/ttyACM0)", prog);
    std::println("  {} --baud <rate>                    Baud rate (default: 115200)", prog);
}

int main(int argc, char* argv[]) {
    std::println("Hand detection and gesture recognition!");

    // --- Parse arguments first, before any hardware init ---
    bool use_camera = true;
    bool use_bluetooth = false;
    std::string images_input;
    float cls_threshold = SCORE_THRESHOLD;
    bool use_serial = false;
    std::string serial_port  = "/dev/ttyUSB0"; // Blue Pill USB-CDC default
    unsigned int serial_baud = 115200;

    for (int i = 1; i < argc; ++i) {
        std::string arg(argv[i]);
        if (arg == "--camera") {
            use_camera = true;
        } else if (arg == "--images" && i + 1 < argc) {
            use_camera = false;
            images_input = argv[++i];
        } else if (arg == "--threshold" && i + 1 < argc) {
            cls_threshold = std::stof(argv[++i]);
        } else if (arg == "--help" || arg == "-h") {
            print_usage(argv[0]);
            return 0;
        } else if (arg == "--serial") {
            use_serial = true;
        } else if (arg == "--serial-port" && i + 1 < argc) {
            serial_port = argv[++i];
            use_serial = true;               // implied
        } else if (arg == "--baud" && i + 1 < argc) {
            serial_baud = static_cast<unsigned int>(std::stoul(argv[++i]));
		} else {
            std::cerr << "Unknown argument: " << arg << "\n";
            print_usage(argv[0]);
            return -1;
        }
    }

    serialib serial;
    if (use_serial) {
        if (serial.openDevice(serial_port.c_str(), serial_baud) != 1) {
            std::cerr << "[Serial] Init failed, continuing without serial.\n";
            use_serial = false;
        }
    }

    // --- Load models (shared by both modes) ---
    std::println("Loading detection model...");
    std::string detection_model_path = "./models/best_hand_det.onnx";
    cv::dnn::Net detection_net;
    load_net(detection_net, detection_model_path);

    std::vector<std::string> class_list = { "hand" };

    std::println("Loading classification model...");
    std::string classification_model_path = "./models/best_gesture_cls.onnx";
    cv::dnn::Net classification_net;
    load_net(classification_net, classification_model_path);

    std::vector<std::string> classification_classes = { "left", "right", "close", "open" };
	std::map<std::string, char> gesture_to_char = {
		{"left", 0x01},
		{"right", 0x02},
		{"close", 0x03},
		{"open", 0x04}
	};

    std::println("Classification confidence threshold: {:.2f}", cls_threshold);

    // MODE A: Image mode
    if (!use_camera) {
        std::vector<fs::path> image_paths = collect_image_paths(images_input);
        if (image_paths.empty()) {
            std::cerr << "No valid images found in: " << images_input << "\n";
            return -1;
        }
        std::println("Found {} image(s). Press any key to advance, 'q' to quit.", image_paths.size());

        for (const auto& img_path : image_paths) {
            cv::Mat frame = cv::imread(img_path.string());
            if (frame.empty()) {
                std::cerr << "Could not read: " << img_path << "\n";
                continue;
            }
            std::println("Processing: {}", img_path.filename().string());
            process_frame(frame, detection_net, classification_net,
                          class_list, classification_classes, cls_threshold);

            cv::imshow("Image - " + img_path.filename().string(), frame);

            int key = cv::waitKey(0);
            if (key == 'q' || key == 'Q') {
                std::println("Quit by user.");
                break;
            }
            cv::destroyAllWindows();
        }
        return 0;
    }

    // MODE B: Camera mode
    cv::VideoCapture capture(0);
    if (!capture.isOpened()) {
        std::cerr << "Error: could not open camera.\n";
        return -1;
    }
    std::println("Camera opened. Press any key to quit.");

	static auto last_send = std::chrono::steady_clock::now();
    cv::Mat frame;
    while (true) {
        capture >> frame;
        if (frame.empty()) break;

        auto gesture = process_frame(frame, detection_net, classification_net,
                      class_list, classification_classes, cls_threshold);

        if (use_serial && serial.isDeviceOpen() && gesture.has_value()) {
			auto now = std::chrono::steady_clock::now();
			if (now - last_send > std::chrono::milliseconds(200)) {
				auto g = gesture.value();
				serial.writeChar(gesture_to_char[g]);
				std::println("Sended gesture: {:02x}", static_cast<unsigned char>(gesture_to_char[g]));
				last_send = now;
            }
        }
        cv::imshow("webcam", frame);

        if (cv::waitKey(30) >= 0) {
            std::println("Finished by user.");
            break;
        }
    }
    return 0;
}

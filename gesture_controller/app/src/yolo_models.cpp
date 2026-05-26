#include "opencv2/core.hpp"
#include <yolo_models.hpp>
#include <print>

void load_net(cv::dnn::Net &net, std::string &model) {
	auto result = cv::dnn::readNet(model);

	std::println("Running on CPU");
	result.setPreferableBackend(cv::dnn::DNN_BACKEND_OPENCV);
	result.setPreferableTarget(cv::dnn::DNN_TARGET_CPU);

	net = result;
}


cv::Mat format_yolov11(const cv::Mat &source) {
	int col = source.cols;
	int row = source.rows;
	int _max = MAX(col, row);
	cv::Mat result = cv::Mat::zeros(_max, _max, CV_8UC3);
	source.copyTo(result(cv::Rect(0, 0, col, row)));
	return result;
}


std::vector<cv::Mat> pre_process(cv::Mat &input_image, cv::dnn::Net &net,
		float width, float height) {
	// Convert to blob.
	cv::Mat blob;
	cv::dnn::blobFromImage(input_image, blob, 1. / 255., cv::Size(width, height), cv::Scalar(), true, false);

	net.setInput(blob);

	// Forward propagate.
	std::vector<cv::Mat> outputs;
	net.forward(outputs, net.getUnconnectedOutLayersNames());

	return outputs;
}

// Detection function
void detect(cv::Mat &image, cv::dnn::Net &net, std::vector<Detection> &output,
			const std::vector<std::string> &className) {
	// Format the input image to fit the model input requirements
	auto input_image = format_yolov11(image);

	std::vector<cv::Mat> outputs = pre_process(input_image, net, DET_INPUT_WIDTH, DET_INPUT_HEIGHT);

	// Scaling factors to map the bounding boxes back to original image size
	float x_factor = input_image.cols / DET_INPUT_WIDTH;
	float y_factor = input_image.rows / DET_INPUT_HEIGHT;

	std::vector<int> class_ids;     // Stores class IDs of detections
	std::vector<float> confidences; // Stores confidence scores of detections
	std::vector<cv::Rect> boxes;    // Stores bounding boxes
	
	float *data = (float *)outputs[0].data;

	const int dimensions = outputs[0].size[1];
	const int rows = outputs[0].size[2];

	// Loop through all the rows to process predictions
	for (int i = 0; i < rows; ++i) {
		// Get the confidence of the current detection
		float confidence = data[4 * rows + i];

		// Process only detections with confidence above the threshold
		if (confidence >= CONFIDENCE_THRESHOLD) {
			// Calculate the bounding box coordinates
			float x = data[0 * rows + i];
			float y = data[1 * rows + i];
			float w = data[2 * rows + i];
			float h = data[3 * rows + i];
			int left = int((x - 0.5 * w) * x_factor);
			int top = int((y - 0.5 * h) * y_factor);
			int width = int(w * x_factor);
			int height = int(h * y_factor);

			boxes.push_back(cv::Rect(left, top, width, height));
			confidences.push_back(confidence);
			class_ids.push_back(0);
		}
	}

	// Apply Non-Maximum Suppression
	std::vector<int> nms_result;
	cv::dnn::NMSBoxes(boxes, confidences, SCORE_THRESHOLD, NMS_THRESHOLD,
					nms_result);

	// Draw the NMS filtered boxes and push results to output
	for (int i = 0; i < nms_result.size(); i++) {
		int idx = nms_result[i];

		// Only push the filtered detections
		Detection result;
		result.class_id = class_ids[idx];
		result.confidence = confidences[idx];
		result.box = boxes[idx];
		output.push_back(result);

		// Draw the final NMS bounding box and label
		cv::rectangle(image, result.box, cv::Scalar(0, 255, 0), 3);
		std::string label = className[result.class_id];
		// cv::putText(image, label, cv::Point(result.box.x, result.box.y - 5),
		// 			cv::FONT_HERSHEY_SIMPLEX, 4, cv::Scalar(0, 0, 255), 4);
	}
}

bool classify(cv::Mat &image, cv::dnn::Net &net, Classification &output,
              const std::vector<std::string> &className) {
    // Safe defaults
    output.class_id = -1;
    output.confidence = 0.0f;

    auto input_image = format_yolov11(image);

    cv::Mat blob;
    cv::dnn::blobFromImage(
        input_image, blob, 1.0 / 255.0,
        cv::Size((int)CLS_INPUT_WIDTH, (int)CLS_INPUT_HEIGHT),
        cv::Scalar(), true, false
    );

    net.setInput(blob);

    cv::Mat out = net.forward();

    std::cout << "cls out.dims = " << out.dims << "  sizes: ";
    for (int i = 0; i < out.dims; ++i) std::cout << out.size[i] << " ";
    std::cout << std::endl;

    cv::Mat scores = out.reshape(1, 1);  // [1 x N]

    if (scores.cols < (int)className.size()) {
        std::println("Classification output has {} values, but {} labels are expected.",
                     scores.cols, className.size());
        return false;
    }

    cv::Point class_id;
    double max_class_score;
    cv::minMaxLoc(scores, nullptr, &max_class_score, nullptr, &class_id);

    // Clamp index just in case
    int idx = std::clamp(class_id.x, 0, (int)className.size() - 1);

    output.class_id = idx;
    output.confidence = (float)max_class_score;

    std::println("id: {}, confidence: {}", output.class_id, output.confidence);

    // If you still want a confidence threshold for using the result, do it in caller.
    return true;
}

#pragma once

#include <opencv2/core/mat.hpp>
#include <opencv2/dnn/dnn.hpp>
#include <opencv2/opencv.hpp>
#include <string>
#include <vector>

const float DET_INPUT_WIDTH = 640.0;
const float DET_INPUT_HEIGHT = 640.0;
const float CLS_INPUT_WIDTH = 640.0;
const float CLS_INPUT_HEIGHT = 640.0;
const float SCORE_THRESHOLD = 0.45;
const float NMS_THRESHOLD = 0.45;
const float CONFIDENCE_THRESHOLD = 0.45;

struct Detection {
  int class_id;
  float confidence;
  cv::Rect box;
};

struct Classification {
  int class_id;
  float confidence;
};

void load_net(cv::dnn::Net &net, std::string &model);

cv::Mat format_yolov11(const cv::Mat &source);

std::vector<cv::Mat> pre_process(cv::Mat &input_image, cv::dnn::Net &net,
		float width, float height);

void detect(cv::Mat &image, cv::dnn::Net &net, std::vector<Detection> &output,
        const std::vector<std::string> &className);

bool classify(cv::Mat &image, cv::dnn::Net &net, Classification &output,
		  const std::vector<std::string> &className);


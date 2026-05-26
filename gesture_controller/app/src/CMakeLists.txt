add_library(yolo_models yolo_models.cpp)
add_library(serialib serialib.cpp)

target_include_directories(yolo_models PUBLIC ${OpenCV_INCLUDE_DIRS} ${PROJECT_SOURCE_DIR}/include)
target_include_directories(serialib PUBLIC ${OpenCV_INCLUDE_DIRS} ${PROJECT_SOURCE_DIR}/include)

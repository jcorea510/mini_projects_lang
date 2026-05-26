cmake_minimum_required(VERSION 4.1)

project(HandController LANGUAGES CXX)

set(CMAKE_CXX_STANDARD 23)
set(CMAKE_CXX_STANDARD_REQUIRED ON)
set(CMAKE_EXPORT_COMPILE_COMMANDS ON)

# find_package(simpleble REQUIRED CONFIG)
find_package(OpenCV REQUIRED)

add_subdirectory(${PROJECT_SOURCE_DIR}/src)

add_executable(${PROJECT_NAME} main.cpp)
add_executable(probe_serial probe_serial.cpp)

target_include_directories(${PROJECT_NAME} PRIVATE ${OpenCV_INCLUDE_DIRS} ${PROJECT_SOURCE_DIR}/include)
target_link_libraries(${PROJECT_NAME} ${OpenCV_LIBS} yolo_models)
target_link_libraries(${PROJECT_NAME} ${OpenCV_LIBS} serialib)

target_include_directories(probe_serial PRIVATE ${OpenCV_INCLUDE_DIRS} ${PROJECT_SOURCE_DIR}/include)
target_link_libraries(probe_serial ${OpenCV_LIBS} serialib)

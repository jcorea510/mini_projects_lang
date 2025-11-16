cmake_minimum_required(VERSION 4.1)

project(circuitx LANGUAGES CXX)

set(CMAKE_CXX_STANDARD 23)
set(CMAKE_CXX_STANDARD_REQUIRED ON)
set(CMAKE_EXPORT_COMPILE_COMMANDS ON)
set(CMAKE_BUILD_TYPE DEBUG)

# adding dependencies from external repositories
include(FetchContent)
set(FETCHCONTENT_UPDATES_DISCONNECTED ON)

FetchContent_Declare(matplotplusplus
        GIT_REPOSITORY https://github.com/alandefreitas/matplotplusplus
        GIT_TAG origin/master) # or whatever tag you want

FetchContent_Declare(json 
	URL https://github.com/nlohmann/json/releases/download/v3.12.0/json.tar.xz)

FetchContent_Declare(
    argparse
    GIT_REPOSITORY https://github.com/p-ranav/argparse.git
)

FetchContent_MakeAvailable(matplotplusplus json argparse)

# adding dependencies from system (requiere building and installation)
find_package(Eigen3 REQUIRED NO_MODULE)

add_executable(
	${PROJECT_NAME}
	src/main.cpp
	src/components.cpp
	src/simulator.cpp
	src/plotter.cpp
	src/netlist.cpp
)

target_include_directories(${PROJECT_NAME} PRIVATE ${PROJECT_SOURCE_DIR}/include)
target_link_libraries(
	${PROJECT_NAME} PRIVATE 
	Eigen3::Eigen 
	matplot
	nlohmann_json::nlohmann_json
	argparse)


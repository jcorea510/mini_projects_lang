# gesture_controller

This is a remote controller based on computer vision
using YOLO models to detect and classify and hand gesture
to execute commands in a MCU STM32 blue pill.

I'll say the trained models are heavy overfitted since 
they was trained with very few data.

![](./docs/close_annotated.jpg)
![](./docs/open_annotated.jpg)

## How to use
First compiles the C++ programm running.

```zsh
cd app/
mkdir build\
cmake -S . -B build
cmake --build build

./build/HandController --camera
```

Then runs the rust program to program the MCU running.

```zsh
cd ble_gesture_controller
cargo run
```

If you want instead to train models you need to install yolo ultralitic,
some YOLO models for classification and detection and then run next commands
in project's root.

```zsh
./train_gesture_classification.sh

./train_hand_detection.sh
```

Of course you will need dataset and labeled images for fine-tunning if needed.


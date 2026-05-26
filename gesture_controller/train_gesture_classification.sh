echo "Creating save directories"
mkdir -p runs/gesture_classification/

echo "Executing YOLO training Experiment 1"
yolo classify train model=models/gesture_classification/yolo11n-cls.pt \
	data=dataset/gesture_classification/ epochs=50 imgsz=640 batch=8 \
	project=runs/gesture_classification/ name=yolo11_gestures


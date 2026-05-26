echo "Creating save directories"
mkdir -p runs/hand_detection/

echo "Executing YOLO training Experiment 1"
yolo detect train model=models/hand_detection/yolo11n.pt \
	data=dataset/hand_detection/data.yaml epochs=50 imgsz=640 batch=8 \
	project=runs/hand_detection/ name=yolo11_hands


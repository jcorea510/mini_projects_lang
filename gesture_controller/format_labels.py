import json
import os
import random
import shutil
import glob
import argparse
from pathlib import Path
from PIL import Image

# ============================================================
# Constants
# ============================================================

# Detection always uses a single class regardless of gesture.
DETECTION_CLASS_ID = 0
DETECTION_CLASS_NAME = "hand"

# Gesture → classification subfolder name.
CLASSIFICATION_DIR = {
    "open":  "hand_open",
    "close": "hand_close",
    "left":  "finger_left",
    "right": "finger_right",
}

VALID_EXTENSIONS = [".jpg", ".jpeg", ".png"]

# Extra margin added around each crop (fraction of box side length).
# Set to 0.0 for a tight crop.
PADDING = 0.10


# ============================================================
# Filesystem helpers
# ============================================================

def ensure_dir(path: str):
    os.makedirs(path, exist_ok=True)


# ============================================================
# LabelMe JSON reader
# ============================================================

def read_label_data(file_path: str) -> dict:
    """Parse a LabelMe JSON file and return a flat dict with all fields needed
    for both the detection and classification pipelines."""
    with open(file_path, "r") as f:
        data = json.load(f)

    shape = data["shapes"][0]
    (x1, y1), (x2, y2) = shape["points"][0], shape["points"][1]

    return {
        "label":        shape["label"],
        "image_height": data["imageHeight"],
        "image_width":  data["imageWidth"],
        "x1": x1, "y1": y1,
        "x2": x2, "y2": y2,
        "image_path":   data["imagePath"],
    }


# ============================================================
# Bounding-box helpers
# ============================================================

def normalize_label_box(x1, y1, x2, y2, image_height, image_width):
    """Convert pixel corner coordinates to YOLO-normalised cx/cy/w/h."""
    left, right  = min(x1, x2), max(x1, x2)
    top,  bottom = min(y1, y2), max(y1, y2)

    width  = right  - left
    height = bottom - top

    x_center = (left + width  / 2) / image_width
    y_center  = (top  + height / 2) / image_height
    w_norm   = width  / image_width
    h_norm   = height / image_height

    return x_center, y_center, w_norm, h_norm


def crop_with_padding(image: Image.Image, x1, y1, x2, y2,
                      padding: float = PADDING) -> Image.Image:
    """Return a padded crop of *image* defined by pixel corners (x1,y1)–(x2,y2).

    *padding* is a fraction of the box's own side lengths, so the margin
    scales naturally with object size.
    """
    W, H = image.size

    left, right  = min(x1, x2), max(x1, x2)
    top,  bottom = min(y1, y2), max(y1, y2)

    pad_x = padding * (right  - left)
    pad_y = padding * (bottom - top)

    crop_x1 = max(0, left  - pad_x)
    crop_y1 = max(0, top   - pad_y)
    crop_x2 = min(W, right  + pad_x)
    crop_y2 = min(H, bottom + pad_y)

    return image.crop((crop_x1, crop_y1, crop_x2, crop_y2))


# ============================================================
# Dataset structure + YAML
# ============================================================

def create_detection_structure(dataset_root: str):
    for split in ("train", "val"):
        ensure_dir(f"{dataset_root}/hand_detection/images/{split}")
        ensure_dir(f"{dataset_root}/hand_detection/labels/{split}")


def create_classification_structure(dataset_root: str):
    for split in ("train", "val"):
        for class_name in CLASSIFICATION_DIR.values():
            ensure_dir(
                f"{dataset_root}/gesture_classification/{split}/{class_name}"
            )


def create_data_yaml(dataset_root: str):
    """Write data.yaml inside hand_detection/ with a single 'hand' class."""
    yaml_content = (
        "path: dataset\n"
        "train: hand_detection/images/train\n"
        "val:   hand_detection/images/val\n"
        "nc: 1\n"
        "names:\n"
        f"  0: {DETECTION_CLASS_NAME}\n"
    )
    yaml_path = os.path.join(dataset_root, "hand_detection", "data.yaml")
    with open(yaml_path, "w") as f:
        f.write(yaml_content)


# ============================================================
# Per-sample writers
# ============================================================

def save_detection_label(label_data: dict, output_label_path: str):
    """Write a YOLO label file. Class ID is always 0 ('hand')."""
    x_center, y_center, width, height = normalize_label_box(
        label_data["x1"], label_data["y1"],
        label_data["x2"], label_data["y2"],
        label_data["image_height"], label_data["image_width"],
    )
    with open(output_label_path, "w") as f:
        f.write(
            f"{DETECTION_CLASS_ID} "
            f"{x_center:.6f} {y_center:.6f} "
            f"{width:.6f} {height:.6f}\n"
        )


def save_classification_crop(image_path: str, label_data: dict,
                              output_image_path: str):
    """Crop the annotated bounding box (with padding) and save it as a JPEG."""
    try:
        img = Image.open(image_path).convert("RGB")
    except Exception as e:
        print(f"  [WARN] Cannot open image {image_path}: {e}")
        return False

    crop = crop_with_padding(
        img,
        label_data["x1"], label_data["y1"],
        label_data["x2"], label_data["y2"],
    )

    if crop.width < 1 or crop.height < 1:
        print(f"  [WARN] Degenerate crop for {image_path}, skipping")
        return False

    # Always save as JPEG for consistency, even if the source was PNG.
    out_path = Path(output_image_path).with_suffix(".jpg")
    out_path.parent.mkdir(parents=True, exist_ok=True)
    crop.save(str(out_path), "JPEG", quality=95)
    return True


# ============================================================
# Dataset split
# ============================================================

def split_dataset(files: list, train_ratio: float = 0.8,
                  seed: int = 42) -> tuple[list, list]:
    random.seed(seed)
    files = files.copy()
    random.shuffle(files)
    split_index = int(len(files) * train_ratio)
    return files[:split_index], files[split_index:]


# ============================================================
# Main processing
# ============================================================

def process_split(label_files: list, image_dir: str,
                  dataset_root: str, split: str):
    saved_det = saved_cls = skipped = 0

    for label_file in label_files:
        label_data = read_label_data(label_file)
        image_name = label_data["image_path"]
        image_stem = Path(image_name).stem

        image_input_path = os.path.join(image_dir, image_name)
        if not os.path.exists(image_input_path):
            print(f"  [WARN] Image not found: {image_input_path}")
            skipped += 1
            continue

        gesture = label_data["label"]
        if gesture not in CLASSIFICATION_DIR:
            print(f"  [WARN] Unknown label '{gesture}' in {label_file}")
            skipped += 1
            continue

        # ── Detection ────────────────────────────────────────────────────────
        det_img_out   = f"{dataset_root}/hand_detection/images/{split}/{image_name}"
        det_label_out = f"{dataset_root}/hand_detection/labels/{split}/{image_stem}.txt"

        shutil.copy(image_input_path, det_img_out)
        save_detection_label(label_data, det_label_out)
        saved_det += 1

        # ── Gesture classification (cropped) ─────────────────────────────────
        class_dir  = CLASSIFICATION_DIR[gesture]
        cls_out    = (
            f"{dataset_root}/gesture_classification"
            f"/{split}/{class_dir}/{image_stem}.jpg"
        )

        if save_classification_crop(image_input_path, label_data, cls_out):
            saved_cls += 1
        else:
            skipped += 1

    print(f"  [{split}] detection labels: {saved_det} | "
          f"classification crops: {saved_cls} | skipped: {skipped}")


def process_dataset(label_dir: str, image_dir: str,
                    dataset_root: str, train_ratio: float = 0.8):
    create_detection_structure(dataset_root)
    create_classification_structure(dataset_root)
    create_data_yaml(dataset_root)

    label_files = glob.glob(os.path.join(label_dir, "*.json"))
    if not label_files:
        print(f"[ERROR] No JSON label files found in: {label_dir}")
        return

    train_files, val_files = split_dataset(label_files, train_ratio)
    print(f"Dataset split → train: {len(train_files)} | val: {len(val_files)}")

    process_split(train_files, image_dir, dataset_root, split="train")
    process_split(val_files,   image_dir, dataset_root, split="val")

    # ── Per-class crop counts ─────────────────────────────────────────────────
    print("\n── GESTURE CLASSIFICATION COUNTS ───────────────────────────")
    for split in ("train", "val"):
        print(f"  [{split}]")
        for class_name in sorted(CLASSIFICATION_DIR.values()):
            class_dir = Path(dataset_root) / "gesture_classification" / split / class_name
            count = len(list(class_dir.glob("*.jpg"))) if class_dir.exists() else 0
            print(f"    {class_name:<15} {count:>5} crops")

    print("\nDataset formatting completed.")
    print(f"Output → {Path(dataset_root).resolve()}")


# ============================================================
# CLI
# ============================================================

def main():
    parser = argparse.ArgumentParser(
        description="Format a LabelMe dataset for YOLO hand detection "
                    "and gesture classification."
    )
    parser.add_argument("--label_dir",    required=True,
                        help="Directory with LabelMe JSON files")
    parser.add_argument("--image_dir",    required=True,
                        help="Directory with source images")
    parser.add_argument("--dataset_root", default="dataset",
                        help="Output root directory (default: dataset)")
    parser.add_argument("--train_ratio",  type=float, default=0.8,
                        help="Fraction of data used for training (default: 0.8)")
    parser.add_argument("--padding",      type=float, default=PADDING,
                        help=f"Crop padding as a fraction of box size "
                             f"(default: {PADDING})")

    args = parser.parse_args()

    # Allow overriding padding via CLI without touching the module constant.
    global PADDING
    PADDING = args.padding

    process_dataset(
        label_dir=args.label_dir,
        image_dir=args.image_dir,
        dataset_root=args.dataset_root,
        train_ratio=args.train_ratio,
    )


if __name__ == "__main__":
    main()

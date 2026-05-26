import torch
from torchvision import transforms
import torch.nn as nn
import torch.nn.functional as F

CLASSES = {
    'aa': 0, 'chi': 1, 'ee': 2, 'fu': 3, 'ha': 4, 'he': 5, 'hi': 6, 'ho': 7,
    'ii': 8, 'ka': 9, 'ke': 10, 'ki': 11, 'ko': 12, 'ku': 13, 'ma': 14,
    'me': 15, 'mi': 16, 'mo': 17, 'mu': 18, 'na': 19, 'ne': 20, 'ni': 21,
    'nn': 22, 'no': 23, 'nu': 24, 'oo': 25, 'ra': 26, 're': 27, 'ri': 28,
    'ro': 29, 'ru': 30, 'sa': 31, 'se': 32, 'shi': 33, 'so': 34, 'su': 35,
    'ta': 36, 'te': 37, 'to': 38, 'tsu': 39, 'uu': 40, 'wa': 41, 'wo': 42,
    'ya': 43, 'yo': 44, 'yu': 45
}

IDX_TO_HIRAGANA = {v: k for k, v in CLASSES.items()}

# Standard transform for validation/testing/inference - no augmentation
TRANSFORM = transforms.Compose(
    [
        transforms.Resize((16, 16)),
        transforms.Grayscale(),
        transforms.ToTensor(),
        transforms.Normalize(mean=(0.5,), std=(0.5,)),
    ]
)

# Augmented transform for training - GENTLE augmentation for small 16x16 images
# Conservative settings to preserve character readability while adding variation
TRANSFORM_AUGMENTED = transforms.Compose(
    [
        transforms.Resize((18, 18)),  # Slightly larger for cropping
        transforms.Grayscale(),
        
        # GENTLE random rotation (±5 degrees max - too much distorts 16x16 characters)
        transforms.RandomRotation(
            degrees=5,
            interpolation=transforms.InterpolationMode.BILINEAR,
            fill=255  # white background
        ),
        
        # GENTLE affine transformations
        transforms.RandomAffine(
            degrees=0,  # rotation handled separately
            translate=(0.05, 0.05),  # max 5% shift (less than before)
            scale=(0.95, 1.05),  # 95-105% scale (narrower range)
            shear=3,  # max 3 degrees shear (much less)
            interpolation=transforms.InterpolationMode.BILINEAR,
            fill=255
        ),
        
        # Random crop back to 16x16
        transforms.RandomCrop(16, padding=1, padding_mode='edge'),
        transforms.ToTensor(),
        
        # MILD brightness/contrast variation (apply less often)
        transforms.RandomApply([
            transforms.ColorJitter(brightness=0.2, contrast=0.2)
        ], p=0.3),  # reduced from 0.5
        
        # VERY MILD blur (only occasionally, very light)
        transforms.RandomApply([
            transforms.GaussianBlur(kernel_size=3, sigma=(0.1, 0.4))  # reduced max sigma
        ], p=0.15),  # reduced from 0.3
        
        # REMOVED: Random erasing - too destructive for small characters
        # REMOVED: Random perspective - too aggressive for 16x16
        
        transforms.Normalize(mean=(0.5,), std=(0.5,)),
    ]
)

# Moderate augmentation - middle ground option
TRANSFORM_AUGMENTED_MODERATE = transforms.Compose(
    [
        transforms.Resize((18, 18)),
        transforms.Grayscale(),
        transforms.RandomRotation(degrees=8, interpolation=transforms.InterpolationMode.BILINEAR, fill=255),
        transforms.RandomAffine(degrees=0, translate=(0.08, 0.08), scale=(0.92, 1.08), shear=5,
                               interpolation=transforms.InterpolationMode.BILINEAR, fill=255),
        transforms.RandomCrop(16, padding=1, padding_mode='edge'),
        transforms.ToTensor(),
        transforms.RandomApply([transforms.ColorJitter(brightness=0.25, contrast=0.25)], p=0.4),
        transforms.RandomApply([transforms.GaussianBlur(kernel_size=3, sigma=(0.1, 0.6))], p=0.2),
        transforms.Normalize(mean=(0.5,), std=(0.5,)),
    ]
)


class HiraganaCNN(nn.Module):
    def __init__(self, channels=1, input_size=16, output_size=46):
        super(HiraganaCNN, self).__init__()
        
        self.conv1 = nn.Conv2d(
            in_channels=channels,
            out_channels=32 * channels,
            kernel_size=3,
            stride=1,
            bias=True,
        )
        self.droput1 = nn.Dropout(0.25)
        self.conv2 = nn.Conv2d(
            in_channels=32 * channels,
            out_channels=32 * 2 * channels,
            kernel_size=3,
            stride=1,
            bias=True,
        )
        self.droput2 = nn.Dropout(0.5)
        
        # Calculate feature size dynamically
        n_features = 0
        with torch.no_grad():
            dummy = torch.zeros(1, channels, input_size, input_size)
            dummy = self.conv1(dummy)
            dummy = self.conv2(dummy)
            dummy = F.max_pool2d(dummy, 2)
            n_features = dummy.numel()

        self.fc1 = nn.Linear(n_features, 128)
        self.fc2 = nn.Linear(128, output_size)

    def forward(self, x):
        x = self.conv1(x)
        x = F.relu(x)
        x = self.conv2(x)
        x = F.relu(x)
        x = F.max_pool2d(x, 2)
        x = self.droput1(x)
        x = torch.flatten(x, 1)
        x = self.fc1(x)
        x = F.relu(x)
        x = self.droput1(x)
        x = self.fc2(x)
        return x

    def load_model(self, model_path, device):
        """Load model weights from file - call this once before predictions"""
        self.load_state_dict(torch.load(model_path, map_location=device))
        self.eval()
        print(f"Model loaded from {model_path}")

    def predict(self, image_tensor):
        """Predict class for image tensor (model must be loaded first)"""
        with torch.inference_mode():
            outputs = self(image_tensor)
            _, predicted_class = torch.max(outputs, 1)
            return predicted_class


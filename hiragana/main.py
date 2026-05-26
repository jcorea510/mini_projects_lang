import customtkinter as ctk
import torch
from PIL import Image
from torchvision import datasets
from torch.utils.data import DataLoader, random_split, Subset
import torch.nn.functional as F
import argparse
import hiragana_cnn
import gui
import matplotlib.pyplot as plt
import numpy as np
import time
from pathlib import Path

def train(log_interval, model, device, train_loader, optimizer, epoch):
    model.train()
    train_loss = 0
    correct = 0
    total = 0
    
    for batch_idx, (data, target) in enumerate(train_loader):
        data, target = data.to(device), target.to(device)
        optimizer.zero_grad()
        output = model(data)
        loss = F.cross_entropy(output, target)
        loss.backward()
        optimizer.step()
        
        train_loss += loss.item() * data.size(0)
        pred = output.argmax(dim=1, keepdim=True)
        correct += pred.eq(target.view_as(pred)).sum().item()
        total += data.size(0)
        
        if batch_idx % log_interval == 0:
            print(
                "Train Epoch: {} [{}/{} ({:.0f}%)]\tLoss: {:.6f}".format(
                    epoch,
                    batch_idx * len(data),
                    len(train_loader.dataset),
                    100.0 * batch_idx / len(train_loader),
                    loss.item(),
                )
            )
    
    avg_loss = train_loss / total
    accuracy = 100.0 * correct / total
    return avg_loss, accuracy


def test(model, device, test_loader, collect_predictions=False):
    model.eval()
    test_loss = 0
    correct = 0
    all_preds = []
    all_targets = []
    
    with torch.no_grad():
        for data, target in test_loader:
            data, target = data.to(device), target.to(device)
            output = model(data)
            test_loss += F.cross_entropy(output, target, reduction="sum").item()
            pred = output.argmax(dim=1, keepdim=True)
            correct += pred.eq(target.view_as(pred)).sum().item()
            
            if collect_predictions:
                all_preds.extend(pred.cpu().numpy().flatten())
                all_targets.extend(target.cpu().numpy())

    test_loss /= len(test_loader.dataset)
    accuracy = 100.0 * correct / len(test_loader.dataset)

    print(
        "\nTest set: Average loss: {:.4f}, Accuracy: {}/{} ({:.0f}%)\n".format(
            test_loss,
            correct,
            len(test_loader.dataset),
            accuracy,
        )
    )
    
    if collect_predictions:
        return test_loss, accuracy, all_preds, all_targets
    return test_loss, accuracy


def plot_training_metrics(metrics, save_dir="training_results"):
    """Plot training metrics: loss and accuracy over epochs"""
    Path(save_dir).mkdir(exist_ok=True)
    
    epochs = range(1, len(metrics['train_loss']) + 1)
    
    # Create figure with subplots
    fig, axes = plt.subplots(2, 2, figsize=(14, 10))
    fig.suptitle('Training Metrics', fontsize=16, fontweight='bold')
    
    # Plot 1: Loss comparison
    ax1 = axes[0, 0]
    ax1.plot(epochs, metrics['train_loss'], 'b-o', label='Train Loss', linewidth=2, markersize=4)
    ax1.plot(epochs, metrics['test_loss'], 'r-s', label='Test Loss', linewidth=2, markersize=4)
    ax1.set_xlabel('Epoch', fontsize=12)
    ax1.set_ylabel('Loss', fontsize=12)
    ax1.set_title('Loss over Epochs', fontsize=13, fontweight='bold')
    ax1.legend(fontsize=10)
    ax1.grid(True, alpha=0.3)
    
    # Plot 2: Accuracy comparison
    ax2 = axes[0, 1]
    ax2.plot(epochs, metrics['train_acc'], 'b-o', label='Train Accuracy', linewidth=2, markersize=4)
    ax2.plot(epochs, metrics['test_acc'], 'r-s', label='Test Accuracy', linewidth=2, markersize=4)
    ax2.set_xlabel('Epoch', fontsize=12)
    ax2.set_ylabel('Accuracy (%)', fontsize=12)
    ax2.set_title('Accuracy over Epochs', fontsize=13, fontweight='bold')
    ax2.legend(fontsize=10)
    ax2.grid(True, alpha=0.3)
    
    # Plot 3: Training time per epoch
    ax3 = axes[1, 0]
    ax3.bar(epochs, metrics['epoch_time'], color='green', alpha=0.7)
    ax3.set_xlabel('Epoch', fontsize=12)
    ax3.set_ylabel('Time (seconds)', fontsize=12)
    ax3.set_title('Training Time per Epoch', fontsize=13, fontweight='bold')
    ax3.grid(True, alpha=0.3, axis='y')
    
    # Plot 4: Summary statistics
    ax4 = axes[1, 1]
    ax4.axis('off')
    summary_text = f"""
    Training Summary
    {'='*40}
    
    Total Epochs: {len(epochs)}
    Total Training Time: {sum(metrics['epoch_time']):.2f}s
    Avg Time per Epoch: {np.mean(metrics['epoch_time']):.2f}s
    
    Final Train Loss: {metrics['train_loss'][-1]:.4f}
    Final Test Loss: {metrics['test_loss'][-1]:.4f}
    
    Final Train Accuracy: {metrics['train_acc'][-1]:.2f}%
    Final Test Accuracy: {metrics['test_acc'][-1]:.2f}%
    
    Best Test Accuracy: {max(metrics['test_acc']):.2f}%
    (Epoch {metrics['test_acc'].index(max(metrics['test_acc'])) + 1})
    """
    ax4.text(0.1, 0.5, summary_text, fontsize=11, verticalalignment='center',
             family='monospace', bbox=dict(boxstyle='round', facecolor='wheat', alpha=0.5))
    
    plt.tight_layout()
    plt.savefig(f"{save_dir}/training_metrics.png", dpi=150, bbox_inches='tight')
    print(f"\n Training metrics plot saved to {save_dir}/training_metrics.png")
    plt.close()


def plot_confusion_matrix(all_targets, all_preds, save_dir="training_results"):
    """Plot confusion matrix"""
    try:
        from sklearn.metrics import confusion_matrix
    except ImportError:
        print("Warning: scikit-learn not installed. Skipping confusion matrix.")
        print("Install with: pip install scikit-learn")
        return
    
    Path(save_dir).mkdir(exist_ok=True)
    
    # Compute confusion matrix
    cm = confusion_matrix(all_targets, all_preds)
    
    # Normalize to percentages
    cm_percent = cm.astype('float') / cm.sum(axis=1)[:, np.newaxis] * 100
    
    # Create figure
    fig, ax = plt.subplots(figsize=(16, 14))
    
    # Plot
    im = ax.imshow(cm_percent, interpolation='nearest', cmap='YlOrRd')
    ax.figure.colorbar(im, ax=ax, label='Accuracy (%)')
    
    # Labels
    classes = [hiragana_cnn.IDX_TO_HIRAGANA[i] for i in range(len(hiragana_cnn.IDX_TO_HIRAGANA))]
    ax.set(xticks=np.arange(cm.shape[1]),
           yticks=np.arange(cm.shape[0]),
           xticklabels=classes,
           yticklabels=classes,
           title='Confusion Matrix (Test Set)',
           ylabel='True Label',
           xlabel='Predicted Label')
    
    # Rotate the tick labels
    plt.setp(ax.get_xticklabels(), rotation=45, ha="right", rotation_mode="anchor")
    
    # Add text annotations for diagonal (correct predictions)
    for i in range(len(classes)):
        text = ax.text(i, i, f'{cm_percent[i, i]:.0f}%',
                      ha="center", va="center", color="darkblue", fontweight='bold', fontsize=7)
    
    plt.tight_layout()
    plt.savefig(f"{save_dir}/confusion_matrix.png", dpi=150, bbox_inches='tight')
    print(f"✓ Confusion matrix saved to {save_dir}/confusion_matrix.png")
    plt.close()
    
    # Print per-class accuracy
    print("\nPer-class Accuracy:")
    print("=" * 50)
    for i, class_name in enumerate(classes):
        acc = cm_percent[i, i]
        print(f"  {class_name:4s}: {acc:6.2f}%")


def visualize_predictions(model, device, test_dataset, num_samples=16, save_dir="training_results"):
    """Visualize random predictions from test set"""
    Path(save_dir).mkdir(exist_ok=True)
    
    model.eval()
    
    # Get random samples
    indices = np.random.choice(len(test_dataset), min(num_samples, len(test_dataset)), replace=False)
    
    fig, axes = plt.subplots(4, 4, figsize=(12, 12))
    fig.suptitle('Random Test Predictions', fontsize=16, fontweight='bold')
    
    for idx, ax in zip(indices, axes.flatten()):
        # Get image and label
        img_tensor, true_label = test_dataset[idx]
        
        # Make prediction
        with torch.no_grad():
            img_batch = img_tensor.unsqueeze(0).to(device)
            output = model(img_batch)
            pred_label = output.argmax(dim=1).item()
        
        # Convert tensor to displayable image
        img_np = img_tensor.cpu().numpy().squeeze()
        
        # Denormalize
        img_np = (img_np * 0.5) + 0.5
        img_np = np.clip(img_np, 0, 1)
        
        # Display
        ax.imshow(img_np, cmap='gray')
        true_char = hiragana_cnn.IDX_TO_HIRAGANA[true_label]
        pred_char = hiragana_cnn.IDX_TO_HIRAGANA[pred_label]
        
        color = 'green' if true_label == pred_label else 'red'
        ax.set_title(f'True: {true_char}\nPred: {pred_char}', 
                    fontsize=11, color=color, fontweight='bold')
        ax.axis('off')
    
    plt.tight_layout()
    plt.savefig(f"{save_dir}/sample_predictions.png", dpi=150, bbox_inches='tight')
    print(f" Sample predictions saved to {save_dir}/sample_predictions.png")
    plt.close()


def evaluate(model, device, test_loader):
    """Evaluate model and compute detailed metrics"""
    model.eval()
    all_preds = []
    all_targets = []

    with torch.no_grad():
        for data, target in test_loader:
            data, target = data.to(device), target.to(device)
            output = model(data)
            pred = output.argmax(dim=1)
            all_preds.extend(pred.cpu().numpy())
            all_targets.extend(target.cpu().numpy())

    # Compute metrics
    try:
        from sklearn.metrics import (
            accuracy_score,
            precision_recall_fscore_support,
            classification_report,
        )

        accuracy = accuracy_score(all_targets, all_preds)
        precision, recall, f1, _ = precision_recall_fscore_support(
            all_targets, all_preds, average="weighted", zero_division=0
        )

        print(f"\n{'='*60}")
        print("EVALUATION METRICS")
        print(f"{'='*60}")
        print(f"Accuracy:  {accuracy:.4f} ({100*accuracy:.2f}%)")
        print(f"Precision: {precision:.4f}")
        print(f"Recall:    {recall:.4f}")
        print(f"F1-Score:  {f1:.4f}")
        print(f"{'='*60}\n")

        # Detailed classification report
        print("Detailed Classification Report:")
        print(
            classification_report(
                all_targets,
                all_preds,
                target_names=[hiragana_cnn.IDX_TO_HIRAGANA[i] for i in range(len(hiragana_cnn.IDX_TO_HIRAGANA))],
                zero_division=0,
            )
        )
    except ImportError:
        print(
            "Warning: scikit-learn not installed. Install with: pip install scikit-learn"
        )
        print(
            f"Basic accuracy: {100 * sum([p == t for p, t in zip(all_preds, all_targets)]) / len(all_preds):.2f}%"
        )


def gui_mode(model, device):
    """Run GUI mode for drawing and predicting hiragana"""
    print("\n" + "=" * 60)
    print("GUI MODE - Interactive Drawing Interface")
    print("=" * 60)
    print("This mode will allow you to draw hiragana characters")
    print("and get real-time predictions from the model.")
    print("=" * 60 + "\n")
    ctk.set_appearance_mode("dark")
    ctk.set_default_color_theme("dark-blue")

    device = torch.device("cpu")

    # load your trained model here
    model = hiragana_cnn.HiraganaCNN().to(device)
    model.load_model("hiragana_cnn.pt", device)

    app = gui.App(model, hiragana_cnn.TRANSFORM, device)
    app.mainloop()

def main():
    parser = argparse.ArgumentParser(description="PyTorch HIRAGANA classifier")
    parser.add_argument(
        "mode",
        choices=["train", "inference", "eval", "gui"],
        help=("Select operation mode: train (train model), "
        "inference (predict single image), "
        "eval (compute metrics), "
        "gui (interactive drawing)"),
    )
    parser.add_argument(
        "--save-model",
        action="store_true",
        help="Save the model after training",
        required=False,
    )
    parser.add_argument(
        "--image-path",
        type=str,
        help="Path to image for inference mode",
        required=False,
    )
    parser.add_argument(
        "--model-path",
        type=str,
        default="hiragana_cnn.pt",
        help="Path to saved model (default: hiragana_cnn.pt)",
        required=False,
    )
    parser.add_argument(
        "--dataset-dir",
        type=str,
        default="hiragana",
        help="Path to dataset directory (default: hiragana)",
        required=False,
    )
    parser.add_argument(
        "--epochs",
        type=int,
        default=100,
        help="Number of training epochs (default: 100)",
        required=False,
    )
    parser.add_argument(
        "--no-augmentation",
        action='store_true',
        help="Disable data augmentation during training",
        required=False,
    )
    parser.add_argument(
        "--augmentation-level",
        type=str,
        choices=["gentle", "moderate"],
        default="gentle",
        help="Augmentation intensity: gentle (default, conservative) or moderate (more aggressive)",
        required=False,
    )
    parser.add_argument(
        "--results-dir",
        type=str,
        default="training_results",
        help="Directory to save training results and visualizations (default: training_results)",
        required=False,
    )
    args = parser.parse_args()

    device_str = "cuda" if torch.cuda.is_available() else "cpu"
    device = torch.device(device_str)
    print(f"Using device: {device}")

    # Initialize model
    model = hiragana_cnn.HiraganaCNN().to(device)

    if args.mode == "train":
        # Data loading for training mode with augmentation
        print(f"Loading dataset from: {args.dataset_dir}")
        
        # Choose transform based on augmentation flag
        train_transform = hiragana_cnn.TRANSFORM if args.no_augmentation else hiragana_cnn.TRANSFORM_AUGMENTED
        
        # Load dataset with no transform first to get structure
        dataset_temp = datasets.ImageFolder(args.dataset_dir)
        print(f"Found {len(dataset_temp)} images across {len(dataset_temp.classes)} classes")
        print(f"Classes: {dataset_temp.class_to_idx}")

        TRAIN_RATIO = 0.8
        total_size = len(dataset_temp)
        train_size = int(TRAIN_RATIO * total_size)
        test_size = total_size - train_size
        
        # Get random indices for splitting
        generator = torch.Generator().manual_seed(42)
        indices = torch.randperm(total_size, generator=generator).tolist()
        train_indices = indices[:train_size]
        test_indices = indices[train_size:]
        
        # Create full datasets with appropriate transforms
        train_full_dataset = datasets.ImageFolder(args.dataset_dir, transform=train_transform)
        test_full_dataset = datasets.ImageFolder(args.dataset_dir, transform=hiragana_cnn.TRANSFORM)
        
        # Create subsets using the indices
        train_dataset = Subset(train_full_dataset, train_indices)
        test_dataset = Subset(test_full_dataset, test_indices)
        
        if args.no_augmentation:
            print(f"Train set: {train_size} images (NO augmentation)")
        else:
            print(f"Train set: {train_size} images (With augmentation)")
        
        print(f"Test set: {test_size} images (no augmentation)")

        train_loader = DataLoader(train_dataset, batch_size=16, shuffle=True)
        test_loader = DataLoader(test_dataset, batch_size=16, shuffle=False)

        optimizer = torch.optim.Adam(model.parameters(), lr=0.005)

        # Metrics tracking
        metrics = {
            'train_loss': [],
            'test_loss': [],
            'train_acc': [],
            'test_acc': [],
            'epoch_time': []
        }

        print(f"\nStarting training for {args.epochs} epochs...")
        for epoch in range(1, args.epochs + 1):
            epoch_start_time = time.time()
            
            # Train
            train_loss, train_acc = train(10, model, device, train_loader, optimizer, epoch)
            
            # Test (with predictions on last epoch for confusion matrix)
            if epoch == args.epochs:
                test_loss, test_acc, all_preds, all_targets = test(model, device, test_loader, collect_predictions=True)
            else:
                test_loss, test_acc = test(model, device, test_loader)
            
            epoch_time = time.time() - epoch_start_time
            
            # Store metrics
            metrics['train_loss'].append(train_loss)
            metrics['test_loss'].append(test_loss)
            metrics['train_acc'].append(train_acc)
            metrics['test_acc'].append(test_acc)
            metrics['epoch_time'].append(epoch_time)

        # Generate visualizations
        print("\n" + "="*60)
        print("Generating training visualizations...")
        print("="*60)
        
        # Create results directory
        results_dir = args.results_dir
        Path(results_dir).mkdir(exist_ok=True)
        
        # Plot training metrics
        plot_training_metrics(metrics, results_dir)
        
        # Plot confusion matrix
        plot_confusion_matrix(all_targets, all_preds, results_dir)
        
        # Visualize random predictions
        visualize_predictions(model, device, test_dataset, num_samples=16, save_dir=results_dir)
        
        print("\n" + "="*60)
        print(" All visualizations complete!")
        print(f" Results saved to '{results_dir}/' directory")
        print("="*60)

        if args.save_model:
            torch.save(model.state_dict(), args.model_path)
            print(f"\nModel saved to {args.model_path}")

    elif args.mode == "inference":
        if not args.image_path:
            print("Error: --image-path is required for inference mode")
            print("Usage: python script.py inference --image-path path/to/image.jpg")
            return

        # Load model once
        model.load_model(args.model_path, device)

        # Process image (no augmentation for inference)
        img = Image.open(args.image_path).convert("RGB")
        img_tensor = hiragana_cnn.TRANSFORM(img)
        img_tensor = img_tensor.unsqueeze(0).to(device)

        # Predict
        prediction = model.predict(img_tensor)
        prediction_idx = prediction.item()
        hiragana = hiragana_cnn.IDX_TO_HIRAGANA[int(prediction_idx)]

        print(f"\nPrediction Results:")
        print(f"  Image: {args.image_path}")
        print(f"  Predicted character: {hiragana}")
        print(f"  Class index: {prediction_idx}")

    elif args.mode == "eval":
        # Load model
        model.load_model(args.model_path, device)

        # Load test dataset (no augmentation)
        print(f"Loading dataset from: {args.dataset_dir}")
        dataset = datasets.ImageFolder(args.dataset_dir, transform=hiragana_cnn.TRANSFORM)

        TRAIN_RATIO = 0.8
        total_size = len(dataset)
        train_size = int(TRAIN_RATIO * total_size)
        test_size = total_size - train_size
        _, test_dataset = random_split(dataset, [train_size, test_size])
        print(f"Evaluating on {test_size} test images...")

        test_loader = DataLoader(test_dataset, batch_size=16, shuffle=False)

        # Evaluate
        evaluate(model, device, test_loader)

    elif args.mode == "gui":
        # Load model
        model.load_model(args.model_path, device)
        gui_mode(model, device)


if __name__ == "__main__":
    main()

import customtkinter as ctk
import tkinter as tk
from PIL import Image, ImageDraw
import torch
import hiragana_cnn

CANVAS_SIZE = 256
MODEL_INPUT_SIZE = 16

class DrawCanvas(ctk.CTkFrame):
    def __init__(self, master):
        super().__init__(master)

        self.canvas = tk.Canvas(self, width=CANVAS_SIZE, height=CANVAS_SIZE, bg="white")
        self.canvas.pack(padx=10, pady=10)

        self.clear_btn = ctk.CTkButton(self, text="Clear", command=self.clear)
        self.clear_btn.pack(pady=(0, 10))

        self.image = Image.new("L", (CANVAS_SIZE, CANVAS_SIZE), 255)
        self.draw = ImageDraw.Draw(self.image)

        self.canvas.bind("<B1-Motion>", self.paint)

    def paint(self, event):
        r = 6
        x, y = event.x, event.y
        self.canvas.create_oval(x-r, y-r, x+r, y+r, fill="black", outline="")
        self.draw.ellipse((x-r, y-r, x+r, y+r), fill=0)

    def clear(self):
        self.canvas.delete("all")
        self.image = Image.new("L", (CANVAS_SIZE, CANVAS_SIZE), 255)
        self.draw = ImageDraw.Draw(self.image)

    def get_tensor(self, transform, device):
        img = self.image.resize((MODEL_INPUT_SIZE, MODEL_INPUT_SIZE))
        img_tensor = transform(img).unsqueeze(0).to(device)
        return img_tensor


class ModelPanel(ctk.CTkFrame):
    def __init__(self, master, predict_callback):
        super().__init__(master)

        self.predict_callback = predict_callback

        self.model_var = ctk.StringVar(value="CNN")

        ctk.CTkLabel(self, text="Select Model").pack(anchor="w", padx=10, pady=(10, 0))

        for name in ["CNN", "SVM", "Transformer"]:
            ctk.CTkRadioButton(
                self,
                text=name,
                variable=self.model_var,
                value=name
            ).pack(anchor="w", padx=10)

        self.predict_btn = ctk.CTkButton(self, text="Predict", command=self.predict)
        self.predict_btn.pack(pady=10)

    def predict(self):
        self.predict_callback(self.model_var.get())


class OutputBox(ctk.CTkFrame):
    def __init__(self, master):
        super().__init__(master)

        self.textbox = ctk.CTkTextbox(self, height=160)
        self.textbox.pack(fill="both", expand=True, padx=10, pady=10)

    def write(self, text):
        self.textbox.delete("1.0", "end")
        self.textbox.insert("end", text)


class App(ctk.CTk):
    def __init__(self, model, transform, device):
        super().__init__()

        self.title("Hiragana Classifier")
        self.geometry("520x700")

        self.model = model
        self.transform = transform
        self.device = device

        self.grid_rowconfigure((0, 1, 2), weight=1)
        self.grid_columnconfigure(0, weight=1)

        self.draw_box = DrawCanvas(self)
        self.draw_box.grid(row=0, column=0, sticky="nsew")

        self.model_panel = ModelPanel(self, self.predict)
        self.model_panel.grid(row=1, column=0, sticky="ew")

        self.output = OutputBox(self)
        self.output.grid(row=2, column=0, sticky="nsew")

        self.log_startup()

    def log_startup(self):
        self.output.write(
            "Using device: cpu\n"
            "Model loaded from hiragana_cnn.pt\n\n"
            "Draw a character and press Predict\n"
        )

    def predict(self, model_name):
        img_tensor = self.draw_box.get_tensor(self.transform, self.device)

        with torch.no_grad():
            logits = self.model(img_tensor)
            idx = logits.argmax(1).item()
            char = hiragana_cnn.IDX_TO_HIRAGANA.get(idx, "unknown")

        self.output.write(
            f"Using device: {self.device}\n"
            f"Model: {model_name}\n\n"
            "Prediction Results:\n"
            f"  Predicted character: {char}\n"
            f"  Class index: {idx}\n"
        )


if __name__ == "__main__":
    ctk.set_appearance_mode("dark")
    ctk.set_default_color_theme("dark-blue")

    device = torch.device("cpu")

    # load your trained model here
    model = hiragana_cnn.HiraganaCNN().to(device)
    model.load_model("hiragana_cnn.pt", device)

    app = App(model, hiragana_cnn.TRANSFORM, device)
    app.mainloop()

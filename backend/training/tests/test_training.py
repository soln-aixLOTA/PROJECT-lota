from ..service import train_model
import os

def test_train_model(tmp_path):
    data_file = tmp_path / "data.txt"
    data_file.write_text("sample data\n")

    model_file = tmp_path / "model.bin"
    train_model(str(data_file), str(model_file))

    assert model_file.exists()
    saved_content = model_file.read_text()
    assert "Model weights: [Placeholder]" in saved_content 
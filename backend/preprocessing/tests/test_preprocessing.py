import os
from ..service import preprocess_data

def test_preprocess_data(tmp_path):
    input_file = tmp_path / "input.txt"
    output_file = tmp_path / "output.txt"

    input_file.write_text("Hello\nWorld\n")
    preprocess_data(str(input_file), str(output_file))

    result = output_file.read_text().strip()
    assert "hello\nworld" == result 
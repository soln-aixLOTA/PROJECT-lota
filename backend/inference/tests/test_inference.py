from ..service import run_inference

def test_run_inference():
    result = run_inference("some input")
    assert result in ["Positive", "Negative", "Neutral"] 
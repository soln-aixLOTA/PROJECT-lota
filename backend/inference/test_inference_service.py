import unittest
from flask import Flask
from inference_service import app

class InferenceServiceTestCase(unittest.TestCase):
    def setUp(self):
        self.app = app.test_client()
        self.app.testing = True

    def test_infer_model(self):
        response = self.app.post('/infer', json={
            'model_name': 'test_model',
            'inputs': {'input_data': [1, 2, 3]}
        })
        self.assertEqual(response.status_code, 200)
        self.assertIn('Model test_model inference completed.', response.get_data(as_text=True))

if __name__ == '__main__':
    unittest.main()
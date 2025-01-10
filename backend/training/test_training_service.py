import unittest
from flask import Flask
from training_service import app

class TrainingServiceTestCase(unittest.TestCase):
    def setUp(self):
        self.app = app.test_client()
        self.app.testing = True

    def test_train_model(self):
        response = self.app.post('/train', json={
            'model_type': 'test_model',
            'dataset_path': '/path/to/dataset',
            'epochs': 5
        })
        self.assertEqual(response.status_code, 200)
        self.assertIn('Model test_model trained for 5 epochs.', response.get_data(as_text=True))

if __name__ == '__main__':
    unittest.main()
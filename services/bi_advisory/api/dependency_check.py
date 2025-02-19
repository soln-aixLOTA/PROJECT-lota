import importlib

dependencies = [
    'fastapi', 'uvicorn', 'python_multipart', 'python_jose', 'passlib',
    'pydantic', 'spacy', 'cymem', 'thinc', 'murmurhash', 'typing_extensions',
    'prophet', 'matplotlib', 'preshed'
]

def check_dependencies():
    for dep in dependencies:
        try:
            importlib.import_module(dep)
            print(f"{dep} successfully imported")
        except ImportError as e:
            print(f"Error importing {dep}: {str(e)}")

if __name__ == "__main__":
    check_dependencies()

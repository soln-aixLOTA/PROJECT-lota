# Example plugin interface
class LotaBotsPlugin:
    def initialize(self, config: Dict[str, Any]) -> None:
        pass

    def process(self, input_data: Any) -> Any:
        pass

    def cleanup(self) -> None:
        pass 
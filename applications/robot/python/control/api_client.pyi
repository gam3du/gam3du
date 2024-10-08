from typing import Any, Union


# A handle to an async operation.
class Handle:
    pass


# A result of an async operation.
class Result:
    def is_done(self) -> bool: ...

    def value(self) -> Union[Any, BaseException]: ...


def message(name: str, *parameter: Any) -> Handle: ...


def poll(name: Handle) -> Result: ...

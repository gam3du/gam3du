from rust_py_module import RustStruct, rust_function, rotate_cube, move_forward, turn_left, turn_right  # type: ignore FIXME the IDE should see the native API somehow
import time


class PythonPerson:
    def __init__(self, name):
        self.name = name


def python_callback():
    python_person = PythonPerson("Peter Python")
    rust_object = rust_function(42, "This is a python string", python_person)
    print("Printing member 'numbers' from rust struct: ", rust_object.numbers)
    rust_object.print_in_rust_from_python()


def take_string(string):
    print("Calling python function from rust with string: " + string)


print("Hello World!")

for x in range(100):
    move_forward()
    time.sleep(1.0)
    move_forward()
    time.sleep(1.0)
    turn_left()
    time.sleep(1.0)
    move_forward()
    time.sleep(1.0)
    turn_left()
    time.sleep(1.0)
    turn_left()
    time.sleep(1.0)
    move_forward()
    time.sleep(1.0)
    # rotate_cube(x)
    # time.sleep(0.020)

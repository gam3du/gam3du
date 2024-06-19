from rust_py_module import move_forward, turn_left  # type: ignore FIXME the IDE should see the native API somehow
import time

for x in range(100):
    move_forward()
    move_forward()
    turn_left()
    move_forward()
    turn_left()
    turn_left()
    move_forward()

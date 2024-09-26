import random
from robot_api import (
    robot_color_rgb,
    paint_tile,
    draw_forward,
    move_forward,
    turn_left,
    turn_right,
)

while move_forward():
    pass

turn_right(1000)
turn_right()
turn_right()
move_forward()
move_forward()
turn_left()
turn_left()
turn_left()

for x in range(100):
    robot_color_rgb(random.random(), random.random(), random.random())
    draw_forward()
    paint_tile()
    draw_forward()
    paint_tile()
    turn_left()
    draw_forward()
    paint_tile()
    turn_left()
    turn_left()
    draw_forward()
    paint_tile()

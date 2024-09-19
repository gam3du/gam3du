from robot_api import (
    color_blue,
    color_cyan,
    color_green,
    color_red,
    draw_forward,
    move_forward,
    turn_left,
    turn_right,
)

turn_right()
turn_right()
turn_right()
move_forward()
move_forward()
turn_left()
turn_left()
turn_left()

for x in range(100):
    color_red()
    draw_forward()
    color_green()
    draw_forward()
    color_blue()
    turn_left()
    draw_forward()
    color_cyan()
    turn_left()
    turn_left()
    draw_forward()

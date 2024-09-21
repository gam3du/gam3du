from robot_api import (
    color_rgb,
    draw_forward,
    move_forward,
    turn_left,
    turn_right,
)

turn_right(1000)
turn_right()
turn_right()
move_forward()
move_forward()
turn_left()
turn_left()
turn_left()

for x in range(100):
    color_rgb(0.8, 0.2, 0.2)
    draw_forward()
    color_rgb(0.2, 0.8, 0.2)
    draw_forward()
    color_rgb(0.2, 0.2, 0.8)
    turn_left()
    draw_forward()
    color_rgb(0.8, 0.8, 0.2)
    turn_left()
    turn_left()
    draw_forward()

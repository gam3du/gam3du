from robot_api import (
    robot_color_rgb,
    paint_tile,
    draw_forward,
    move_forward,
    turn_left,
    turn_right,
)

def fahre_gegen_die_wand():
    while draw_forward():
        pass

def beweg_dich (r,g,b):
    robot_color_rgb(r,g,b)
    paint_tile()
    move_forward()

for x in range(4):
    turn_left()

for x in range(4):
    move_forward()

for x in range(4):
    turn_right()

beweg_dich(1,0.1,0.1)
beweg_dich(1,0.5,0)
beweg_dich(1,1,0)
beweg_dich(0.7,1,0.3)
beweg_dich(0.1,1,0.1)
beweg_dich(0.3,0.7,1)
beweg_dich(0.1,0.1,1)
beweg_dich(0.5,0,1)
robot_color_rgb(1,1,1)

turn_left()
turn_left()
draw_forward()
turn_left()
turn_left()
fahre_gegen_die_wand()
turn_left()
turn_left()
draw_forward()
draw_forward()
turn_left()
turn_left()
fahre_gegen_die_wand()
turn_left()
turn_left()
draw_forward()
turn_right()
turn_right()
robot_color_rgb(0,0,0)

while True:
    turn_right(1000/80)

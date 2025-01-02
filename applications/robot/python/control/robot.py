from robot_control_api import (
    robot_color_rgb,
    paint_tile,
    draw_forward,
    move_forward,
    turn_left,
    turn_right,
    set_height,
)

def gehe(wie_oft,farbe):
    for schritt in range(wie_oft):
        robot_color_rgb(farbe/10,farbe/10,1)
        paint_tile()
        move_forward(100)


for x in range(1,11,2):
    gehe(x,x)
    turn_right(100)
    turn_right(100)
    gehe(x,x)
    turn_right(100)
    turn_right(100)
    gehe(x+1,x)
    turn_right(100)
    turn_right(100)
    gehe(x+1,x)
    turn_right(100)
    turn_right(100)










# def turn_left_90():
#     drehe_links(2)

# def umdrehen():
#     drehe_links(4)

# def drehe_links(wie_oft):
#     for x in range(wie_oft):
#         turn_left()

# def fahre_gegen_die_wand():
#     while draw_forward():
#         pass

# def umrandung ():
#     for x in range(9):
#         draw_forward()

# def treppe (height):
#     move_forward()
#     set_height(height)

# def beweg_dich (r,g,b):
#     set_height(1.0)
#     robot_color_rgb(r,g,b)
#     paint_tile()
#     move_forward()

# ##############################################################################

# # In Startposition bringen
# umdrehen()
# for x in range(4):
#     move_forward()
# umdrehen()

# # male Regenbogenbrücke
# beweg_dich(1,0.1,0.1)
# beweg_dich(1,0.5,0)
# beweg_dich(1,1,0)
# beweg_dich(0.7,1,0.3)
# beweg_dich(0.1,1,0.1)
# beweg_dich(0.3,0.7,1)
# beweg_dich(0.1,0.1,1)
# beweg_dich(0.5,0,1)
# robot_color_rgb(1,1,1)

# # Male Umrandung
# turn_left_90()
# draw_forward()
# turn_left_90()
# umrandung()
# turn_left_90()
# draw_forward()
# draw_forward()
# turn_left_90()
# umrandung()
# turn_left_90()
# draw_forward()

# # Male Treppe
# drehe_links(2)
# for x in range(8):
#     treppe(1.0 - x * 0.1)  # geld → kg

# # Werde verrückt
# move_forward()
# robot_color_rgb(0,0,0)
# while True:
#     turn_right(1000/80)

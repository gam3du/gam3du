from api_server import (
    send_boolean_response
)

from robot_plugin_api import (
    draw_forward, move_forward, paint_tile, robot_color_rgb, turn_left, turn_right
)

def on_move_forward(request_id, duration):
    print("on_move_forward", request_id, duration)
    send_boolean_response("robot control", request_id, True)

def on_draw_forward(request_id, duration):
    print("on_draw_forward", request_id, duration)
    send_boolean_response("robot control", request_id, True)

def on_turn_left(request_id, duration):
    print("on_turn_left", request_id, duration)
    send_boolean_response("robot control", request_id, True)

def on_turn_right(request_id, duration):
    print("on_turn_right", request_id, duration)
    send_boolean_response("robot control", request_id, True)

def on_robot_color_rgb(request_id, red, green, blue):
    print("on_robot_color_rgb", request_id, red, green, blue)
    send_boolean_response("robot control", request_id, True)

def on_paint_tile(request_id):
    print("on_paint_tile", request_id)
    send_boolean_response("robot control", request_id, True)

from api_server import (
    send_boolean_response
)

from robot_plugin_api import (
    move_forward, paint_tile, robot_color_rgb, turn, log_trace, log_debug, log_error, log_info, log_warn, set_height, jump
)

log_info("robot_plugin.py loaded")

current_command = 0

def on_robot_stopped():
    global current_command
    log_trace("on_robot_stopped(%s)" % hex(current_command))

    send_boolean_response("robot control", current_command, True)
    log_trace("resetting command(%s)" % hex(current_command))
    current_command = 0

def on_set_height(request_id, height):
    global current_command
    log_trace("on_set_height(%s, %s)" % (hex(request_id), height))
    if current_command:
        log_error("pending command")

    set_height(height)
    send_boolean_response("robot control", request_id, True)

def on_move_forward(request_id, duration):
    global current_command
    log_trace("on_move_forward(%s, %s)" % (hex(request_id), duration))
    if current_command:
        log_error("pending command")

    if not move_forward(False, duration):
        send_boolean_response("robot control", request_id, False)
    else:
        current_command = request_id

def on_jump(request_id, duration):
    global current_command
    log_trace("on_jump(%s, %s)" % (hex(request_id), duration))
    if current_command:
        log_error("pending command")

    if not jump(duration):
        send_boolean_response("robot control", request_id, False)
    else:
        current_command = request_id

def on_draw_forward(request_id, duration):
    global current_command
    log_trace("on_draw_forward(%s, %s)" % (hex(request_id), duration))
    if current_command:
        log_error("pending command")

    if not move_forward(True, duration):
        send_boolean_response("robot control", request_id, False)
    else:
        current_command = request_id

def on_turn_left(request_id, duration):
    global current_command
    log_trace("on_turn_left(%s, %s)" % (hex(request_id), duration))
    if current_command:
        log_error("pending command")

    turn(1, duration)
    current_command = request_id

def on_turn_right(request_id, duration):
    global current_command
    log_trace("on_turn_right(%s, %s)" % (hex(request_id), duration))
    if current_command:
        log_error("pending command")

    turn(-1, duration)
    current_command = request_id

def on_robot_color_rgb(request_id, red, green, blue):
    global current_command
    log_trace("on_robot_color_rgb(%s, [%s, %s, %s])" % (hex(request_id), red, green, blue))
    if current_command:
        log_error("pending command")

    robot_color_rgb(red, green, blue)
    send_boolean_response("robot control", request_id, True)

def on_paint_tile(request_id):
    global current_command
    log_trace("on_paint_tile(%s)" % hex(request_id))
    if current_command:
        log_error("pending command")

    paint_tile()
    send_boolean_response("robot control", request_id, True)

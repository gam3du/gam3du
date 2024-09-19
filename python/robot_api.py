import robot_api_internal

def move_forward(duration=1000):
	robot_api_internal.message("move forward", duration)

def draw_forward(duration=1000):
	robot_api_internal.message("draw forward", duration)

def turn_left(duration=1000):
	robot_api_internal.message("turn left", duration)

def turn_right(duration=1000):
	robot_api_internal.message("turn right", duration)

def color_black():
	robot_api_internal.message("color black")

def color_red():
	robot_api_internal.message("color red")

def color_green():
	robot_api_internal.message("color green")

def color_yellow():
	robot_api_internal.message("color yellow")

def color_blue():
	robot_api_internal.message("color blue")

def color_magenta():
	robot_api_internal.message("color magenta")

def color_cyan():
	robot_api_internal.message("color cyan")

def color_white():
	robot_api_internal.message("color white")


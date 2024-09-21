import robot_api_internal

def move_forward(duration=500):
	robot_api_internal.message("move forward", duration)

def turn_left(duration=300):
	robot_api_internal.message("turn left", duration)

def turn_right(duration=300):
	robot_api_internal.message("turn right", duration)

def color_rgb(red, green, blue):
	robot_api_internal.message("color rgb", red, green, blue)

def draw_forward(duration=500):
	robot_api_internal.message("draw forward", duration)


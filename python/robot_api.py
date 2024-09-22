import robot_api_internal

def turn_left(duration: int = 300):
	robot_api_internal.message("turn left", duration)

def move_forward(duration: int = 500):
	robot_api_internal.message("move forward", duration)

def color_rgb(red: float, green: float, blue: float):
	robot_api_internal.message("color rgb", red, green, blue)

def turn_right(duration: int = 300):
	robot_api_internal.message("turn right", duration)

def draw_forward(duration: int = 500):
	robot_api_internal.message("draw forward", duration)


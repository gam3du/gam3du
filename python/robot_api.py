import robot_api_internal

def move_forward(duration: int = 500) -> bool:
	robot_api_internal.message("move forward", duration)

def draw_forward(duration: int = 500) -> bool:
	robot_api_internal.message("draw forward", duration)

def turn_left(duration: int = 300):
	robot_api_internal.message("turn left", duration)

def turn_right(duration: int = 300):
	robot_api_internal.message("turn right", duration)

def color_rgb(red: float, green: float, blue: float):
	robot_api_internal.message("color rgb", red, green, blue)


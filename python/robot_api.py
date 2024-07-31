import robot_api_internal

def move_forward(duration=1000):
	robot_api_internal.message("move forward", duration)

def turn_left(duration=1000):
	robot_api_internal.message("turn left", duration)

def turn_right(duration=1000):
	robot_api_internal.message("turn right", duration)


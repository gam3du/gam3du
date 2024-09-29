# This file has been generated automatically and shall not be edited by hand!
# generator: launchers/robot/build.rs
# api descriptor: control.api.json

import api_client

def move_forward(duration: int = 500) -> bool:
	return api_client.message("move forward", duration)

def draw_forward(duration: int = 500) -> bool:
	return api_client.message("draw forward", duration)

def turn_left(duration: int = 300):
	return api_client.message("turn left", duration)

def turn_right(duration: int = 300):
	return api_client.message("turn right", duration)

def robot_color_rgb(red: float, green: float, blue: float):
	return api_client.message("robot color rgb", red, green, blue)

def paint_tile():
	return api_client.message("paint tile")


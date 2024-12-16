# This file has been generated automatically and shall not be edited by hand!
# generator: applications/robot/build.rs
# api descriptor: control.api.json

import robot_control_api_async

def set_height(height: float):
	future = robot_control_api_async.set_height(height)
	return future

def move_forward(duration: int = 500) -> bool:
	future = robot_control_api_async.move_forward(duration)
	return future

def jump(duration: int = 500) -> bool:
	future = robot_control_api_async.jump(duration)
	return future

def draw_forward(duration: int = 500) -> bool:
	future = robot_control_api_async.draw_forward(duration)
	return future

def turn_left(duration: int = 300):
	future = robot_control_api_async.turn_left(duration)
	return future

def turn_right(duration: int = 300):
	future = robot_control_api_async.turn_right(duration)
	return future

def robot_color_rgb(red: float, green: float, blue: float):
	future = robot_control_api_async.robot_color_rgb(red, green, blue)
	return future

def paint_tile():
	future = robot_control_api_async.paint_tile()
	return future



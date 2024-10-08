# This file has been generated automatically and shall not be edited by hand!
# generator: launchers/robot/build.rs
# api descriptor: control.api.json

import robot_control_api_async
import asyncio

def move_forward(duration: int = 500) -> bool:
	future = robot_control_api_async.move_forward(duration)
	return asyncio.run(future)

def draw_forward(duration: int = 500) -> bool:
	future = robot_control_api_async.draw_forward(duration)
	return asyncio.run(future)

def turn_left(duration: int = 300):
	future = robot_control_api_async.turn_left(duration)
	return asyncio.run(future)

def turn_right(duration: int = 300):
	future = robot_control_api_async.turn_right(duration)
	return asyncio.run(future)

def robot_color_rgb(red: float, green: float, blue: float):
	future = robot_control_api_async.robot_color_rgb(red, green, blue)
	return asyncio.run(future)

def paint_tile():
	future = robot_control_api_async.paint_tile()
	return asyncio.run(future)



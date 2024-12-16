# This file has been generated automatically and shall not be edited by hand!
# generator: applications/robot/build.rs
# api descriptor: control.api.json

import api_client

def set_height(height: float):
	handle = api_client.message("set height", height)
	while True:
		result = api_client.poll(handle)
		if result.is_done():
			return result.get_value()
		# await asyncio.sleep(0.01)


def move_forward(duration: int = 500) -> bool:
	handle = api_client.message("move forward", duration)
	while True:
		result = api_client.poll(handle)
		if result.is_done():
			return result.get_value()
		# await asyncio.sleep(0.01)


def jump(duration: int = 500) -> bool:
	handle = api_client.message("jump", duration)
	while True:
		result = api_client.poll(handle)
		if result.is_done():
			return result.get_value()
		# await asyncio.sleep(0.01)


def draw_forward(duration: int = 500) -> bool:
	handle = api_client.message("draw forward", duration)
	while True:
		result = api_client.poll(handle)
		if result.is_done():
			return result.get_value()
		# await asyncio.sleep(0.01)


def turn_left(duration: int = 300):
	handle = api_client.message("turn left", duration)
	while True:
		result = api_client.poll(handle)
		if result.is_done():
			return result.get_value()
		# await asyncio.sleep(0.01)


def turn_right(duration: int = 300):
	handle = api_client.message("turn right", duration)
	while True:
		result = api_client.poll(handle)
		if result.is_done():
			return result.get_value()
		# await asyncio.sleep(0.01)


def robot_color_rgb(red: float, green: float, blue: float):
	handle = api_client.message("robot color rgb", red, green, blue)
	while True:
		result = api_client.poll(handle)
		if result.is_done():
			return result.get_value()
		# await asyncio.sleep(0.01)


def paint_tile():
	handle = api_client.message("paint tile")
	while True:
		result = api_client.poll(handle)
		if result.is_done():
			return result.get_value()
		# await asyncio.sleep(0.01)




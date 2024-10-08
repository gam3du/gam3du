import asyncio
import robot_api_internal

async def move_forward(duration: int = 500) -> bool:
	handle = robot_api_internal.message("move forward", duration)
	while True:
		result = robot_api_internal.poll(handle)
		if result.is_done():
			return result.get_value()
		await asyncio.sleep(0.01)


async def draw_forward(duration: int = 500) -> bool:
	handle = robot_api_internal.message("draw forward", duration)
	while True:
		result = robot_api_internal.poll(handle)
		if result.is_done():
			return result.get_value()
		await asyncio.sleep(0.01)


async def turn_left(duration: int = 300):
	handle = robot_api_internal.message("turn left", duration)
	while True:
		result = robot_api_internal.poll(handle)
		if result.is_done():
			return result.get_value()
		await asyncio.sleep(0.01)


async def turn_right(duration: int = 300):
	handle = robot_api_internal.message("turn right", duration)
	while True:
		result = robot_api_internal.poll(handle)
		if result.is_done():
			return result.get_value()
		await asyncio.sleep(0.01)


async def robot_color_rgb(red: float, green: float, blue: float):
	handle = robot_api_internal.message("robot color rgb", red, green, blue)
	while True:
		result = robot_api_internal.poll(handle)
		if result.is_done():
			return result.get_value()
		await asyncio.sleep(0.01)


async def paint_tile():
	handle = robot_api_internal.message("paint tile")
	while True:
		result = robot_api_internal.poll(handle)
		if result.is_done():
			return result.get_value()
		await asyncio.sleep(0.01)



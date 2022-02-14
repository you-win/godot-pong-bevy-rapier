extends StaticBody2D

enum Side {
	NONE,
	LEFT,
	RIGHT
}

export(Side) var side: int = Side.NONE
export var speed: float = 10.0

var move_up_key: int = KEY_SPACE
var move_down_key: int = KEY_BACKSPACE

###############################################################################
# Builtin functions                                                           #
###############################################################################

func _ready() -> void:
	add_to_group("paddle")
	match side:
		Side.LEFT:
			move_up_key = KEY_W
			move_down_key = KEY_S
		Side.RIGHT:
			move_up_key = KEY_UP
			move_down_key = KEY_DOWN

func _physics_process(delta: float) -> void:
	if Input.is_key_pressed(move_up_key):
		position.y -= speed
	if Input.is_key_pressed(move_down_key):
		position.y += speed

###############################################################################
# Connections                                                                 #
###############################################################################

###############################################################################
# Private functions                                                           #
###############################################################################

###############################################################################
# Public functions                                                            #
###############################################################################

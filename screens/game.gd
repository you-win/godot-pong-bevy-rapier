extends Node2D

var ecs

var loader = preload("res://addons/gdnative-runtime-loader/gdnative_runtime_loader.gd").new()

###############################################################################
# Builtin functions                                                           #
###############################################################################

func _ready() -> void:
	loader.presetup()
	loader.setup()
	
	var ecs_factory = loader.create_class("rust-ecs", "EcsFactory")
	
	ecs = ecs_factory.new_ecs()
	
	add_child(ecs)

func _input(event: InputEvent) -> void:
	if event is InputEventKey:
		if event.scancode == KEY_SPACE and event.pressed:
			get_tree().reload_current_scene()

###############################################################################
# Connections                                                                 #
###############################################################################

###############################################################################
# Private functions                                                           #
###############################################################################

###############################################################################
# Public functions                                                            #
###############################################################################

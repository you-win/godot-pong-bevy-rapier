extends Node

const ENV_VAR_NAME: String = "GODOT_ENV"
const ENVS: Dictionary = {
	"DEFAULT": "default",
	"TEST": "test"
}

const Groups: Dictionary = {
	"NONE": "NONE",
	"COMBAT_ENTITY": "COMBAT_ENTITY"
}

var loader: BaseLoader

onready var logger: Logger = load("res://utils/logger.gd").new()

var env: String = ENVS.DEFAULT

###############################################################################
# Builtin functions                                                           #
###############################################################################

func _ready() -> void:
	self.connect("tree_exiting", self, "_on_tree_exiting")

	var system_env = OS.get_environment(ENV_VAR_NAME)
	if system_env:
		env = system_env
	
	if OS.has_feature("HTML5"):
		loader = load("res://utils/loaders/blocking_loader.gd").new()
	else:
		loader = load("res://utils/loaders/background_loader.gd").new()

###############################################################################
# Connections                                                                 #
###############################################################################

func _on_tree_exiting() -> void:
	if env != ENVS.TEST:
		pass
	
	logger.info("Exiting. おやすみ。")

###############################################################################
# Private functions                                                           #
###############################################################################

###############################################################################
# Public functions                                                            #
###############################################################################

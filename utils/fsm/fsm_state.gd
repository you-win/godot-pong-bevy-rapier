class_name FSMState
extends Object

var name: String

var obj: Node
var fsm

func _to_string():
	return name

func on_enter() -> void:
	pass

func run(_delta: float) -> void:
	pass

func on_exit() -> void:
	pass

class_name FSM
extends Object

var obj: Node

var states: Dictionary

var last_state: FSMState
var current_state: FSMState
var next_state: FSMState

###############################################################################
# Builtin functions                                                           #
###############################################################################

func _init(p_obj: Node, p_states: Dictionary, starting_state: FSMState) -> void:
	obj = p_obj
	obj.connect("tree_exiting", self, "_cleanup")
	states = p_states
	
	for state_key in states.keys():
		var state: FSMState = states[state_key]
		state.name = state_key
		state.fsm = self
		state.obj = obj
	
	last_state = starting_state
	current_state = starting_state
	next_state = starting_state

func _to_string():
	return JSON.print(states, "\t")

###############################################################################
# Connections                                                                 #
###############################################################################

###############################################################################
# Private functions                                                           #
###############################################################################

func _cleanup() -> void:
	for i in states.size():
		states.values()[i].free()
	call_deferred("free")

###############################################################################
# Public functions                                                            #
###############################################################################

func run(delta: float) -> void:
	if current_state != next_state:
		current_state.on_exit()
		next_state.on_enter()
		last_state = current_state
		current_state = next_state
	current_state.run(delta)

func switch_state_now(delta: float, state: FSMState) -> void:
	next_state = state
	run(delta)

func switch_state_deferred(state: FSMState) -> void:
	next_state = state

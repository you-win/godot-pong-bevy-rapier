extends RigidBody2D

var last_sign: int = 0

###############################################################################
# Builtin functions                                                           #
###############################################################################

func _ready() -> void:
	var force: float
	if randf() > 0.5:
		force = 500
	else:
		force = -500
	apply_central_impulse(Vector2(force, 0))
	last_sign = sign(force)
	
func _physics_process(delta: float) -> void:
	var colliding_bodies: Array = get_colliding_bodies()
	if colliding_bodies.size() > 0:
		var body = colliding_bodies[0]
		if body.is_in_group("paddle"):
			last_sign = -last_sign
			apply_central_impulse(Vector2(rand_range(100, 200) * last_sign, rand_range(-200, 200)))
		

###############################################################################
# Connections                                                                 #
###############################################################################

###############################################################################
# Private functions                                                           #
###############################################################################

###############################################################################
# Public functions                                                            #
###############################################################################

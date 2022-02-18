extends Node2D

var ecs

var tex

var rid

var tran

###############################################################################
# Builtin functions                                                           #
###############################################################################

func _ready() -> void:
	var ecs_factory = load("res://rust/ecs_factory.gdns").new()
	ecs = ecs_factory.new_ecs()

	add_child(ecs)

#	var rust_sprite = ecs_factory.new_rust_sprite()
#	add_child(rust_sprite)

#	var test = ResourceLoader.load("res://assets/icon.png", "image", false)
#	print(test)

#	var tex := Image.new()
#	tex.load("res://assets/Paddle.png")
#
#	rid = VisualServer.canvas_item_create()
#
#	var tex_rid = VisualServer.texture_create_from_image(tex)
#	tran = Transform2D()
#	tran.origin.x = -100
#	VisualServer.canvas_item_add_texture_rect(rid, Rect2(tex.get_size() / 2, tex.get_size()), tex_rid)
#	VisualServer.canvas_item_set_transform(rid, tran)
#	VisualServer.canvas_item_set_parent(rid, get_canvas_item())

#func _process(delta):
#	if Input.is_key_pressed(KEY_W):
#		tran.origin.y -= 10.0
#		VisualServer.canvas_item_set_transform(rid, tran)
#		print(tran)
#	if Input.is_key_pressed(KEY_S):
#		tran.origin.y += 10.0
#		VisualServer.canvas_item_set_transform(rid, tran)

###############################################################################
# Connections                                                                 #
###############################################################################

###############################################################################
# Private functions                                                           #
###############################################################################

###############################################################################
# Public functions                                                            #
###############################################################################

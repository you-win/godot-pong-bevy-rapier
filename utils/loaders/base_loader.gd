class_name BaseLoader
extends Reference

"""
Helper class for loading resources. Needs to have its `begin_load` function implemented
depending on the loading logic required. e.g. Running on a thread or blocking the main thread
"""

enum LoaderErrors {
	NONE = 400,
	
	INVALID_LOADABLE
}

class Payload extends Object:
	"""
	Loadables are keys to resource paths to be loaded.
	Results are keys to a loadable's loaded + instanced/created object
	
	The calling method should keep a reference to this object and extract
	the results after loading is finished
	"""
	var has_error := false
	var code := OK
	
	var loadables: Dictionary = {} # Either String: Node or String: Array[Node]
	var results: Dictionary = {} # Same as loadables
	
	func free() -> void:
		loadables.clear()
		results.clear()
		.free()

###############################################################################
# Builtin functions                                                           #
###############################################################################

###############################################################################
# Connections                                                                 #
###############################################################################

###############################################################################
# Private functions                                                           #
###############################################################################

func _load(data: Payload) -> void:
	"""
	The primary load function. Iterates through a Payload's loadables and places
	the loaded objects into Payload.results under the same key
	"""
	for key in data.loadables:
		var d = data.loadables[key]
		var v
		match typeof(d):
			TYPE_ARRAY:
				v = []
				for datum in d:
					v.append(_instance_or_new(load(datum)))
			TYPE_STRING:
				v = _instance_or_new(load(d))
			_:
				v = LoaderErrors.INVALID_LOADABLE
		data.results[key] = v
	return

static func _instance_or_new(v):
	"""
	Helper function for instancing or calling new on a resource
	"""
	if v is PackedScene:
		return v.instance()
	elif v is GDScript:
		return v.new()
	else:
		AppManager.logger.error("Tried to load invalid resource: %s" % str(v))
		return v

###############################################################################
# Public functions                                                            #
###############################################################################

func begin_load(_data: Payload) -> void:
	AppManager.logger.error("begin_load is unimplemented!")
	AppManager.logger.trace(":sadface:")

func is_finished() -> bool:
	return false

static func new_payload() -> Payload:
	"""
	Helper function for creating a new Payload
	"""
	return Payload.new()

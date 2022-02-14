class_name Logger
extends Reference

signal message_logged(message)

enum LogType { NONE, INFO, DEBUG, TRACE, ERROR }

var parent_name: String = "DEFAULT_LOGGER"

###############################################################################
# Builtin functions                                                           #
###############################################################################

###############################################################################
# Connections                                                                 #
###############################################################################

###############################################################################
# Private functions                                                           #
###############################################################################

func _log(message: String, log_type: int) -> void:
	var datetime: Dictionary = OS.get_datetime()
	message = "%s %s-%s-%s_%s:%s:%s %s" % [
		parent_name,
		datetime["year"],
		datetime["month"],
		datetime["day"],
		datetime["hour"],
		datetime["minute"],
		datetime["second"],
		message
	]
	
	match log_type:
		LogType.INFO:
			message = "[INFO] %s" % message
		LogType.DEBUG:
			message = "[DEBUG] %s" % message
		LogType.TRACE:
			message = "[TRACE] %s" % message
			var stack_trace: Array = get_stack()
			for i in stack_trace.size() - 2:
				var data: Dictionary = stack_trace[i + 2]
				message = "%s\n\t%d - %s:%d - %s" % [
					message, i, data["source"], data["line"], data["function"]]
		LogType.ERROR:
			message = "[ERROR] %s" % message
			assert(false, message)

	print(message)
	emit_signal("message_logged", message)

###############################################################################
# Public functions                                                            #
###############################################################################

func setup(n) -> void:
	"""
	Initialize the logger with the containing object name. Prefer user-defined values but
	also try to intelligently get the calling object name as well
	"""
	if typeof(n) == TYPE_STRING:
		parent_name = n
	elif n.get_script():
		parent_name = n.get_script().resource_path.get_file()
	elif n.get("name"):
		parent_name = n.name
	else:
		trace("Unable to setup logger using var: %s" % str(n))
	
	while AppManager.sb == null:
		yield(AppManager.get_tree(), "idle_frame")
	
	

func info(message: String) -> void:
	_log(message, LogType.INFO)

func debug(message: String) -> void:
	if OS.is_debug_build():
		_log(message, LogType.DEBUG)

func trace(message: String) -> void:
	if OS.is_debug_build():
		_log(message, LogType.TRACE)

func error(message: String) -> void:
	_log(message, LogType.ERROR)

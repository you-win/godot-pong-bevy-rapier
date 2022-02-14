class_name SignalBroadcaster
extends Reference

func register(o: Object, signal_name: String) -> void:
	connect(signal_name, o, "_on_%s" % signal_name)

static func register_same(o: Object, signal_name: String) -> void:
	o.call("connect", signal_name, o, "_on_%s" % signal_name)

#region Logger

func register_logger(l: Logger) -> void:
	l.connect("message_logged", self, "_on_message_logged")

signal message_logged(text)
func _on_message_logged(text: String) -> void:
	"""
	Aggregate and rebroadcast logger messages to be picked up by gui
	"""
	emit_signal("message_logged", text)

#endregion

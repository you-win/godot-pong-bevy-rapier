extends BaseLoader

"""
For use in desktop builds. Runs the loading logic on a background thread.
"""

var thread: Thread

###############################################################################
# Builtin functions                                                           #
###############################################################################

###############################################################################
# Connections                                                                 #
###############################################################################

###############################################################################
# Private functions                                                           #
###############################################################################

###############################################################################
# Public functions                                                            #
###############################################################################

func begin_load(data: Payload) -> void:
	thread = Thread.new()
	if thread.start(self, "_load", data) != OK:
		AppManager.logger.error("Unable to start loader")

func is_finished() -> bool:
	if not thread.is_alive():
		thread.wait_to_finish()
		thread = null
		return true
	return false

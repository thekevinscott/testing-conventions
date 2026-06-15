# testing-conventions:waiver(coverage): thin launcher shim; its branches are exercised only by the e2e suite, never the unit suite
def launch(flag):
    if flag:
        return "started"
    return "stopped"

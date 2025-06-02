# IMPLEMENTATION_NOTES (Human inserted)

This document contains notes on the implementation of the Helix sub-module plugin for VIM inside Zed.

## Implementation Details

Attempts to re-use the existing Vim code have run into issues due to the
fundamental difference in Vim motions and Helix motions.

For example, when using match brackets, the Vim code forces a return to NORMAL mode when the selection match is done because it's designed with verb + object.
One the object has been selected, then it forces the switch because the motion is complete. Whereas in Helix, the motion starts with the object selection and then the verb is applied.

Several different attempts have been done to re-use the existing Vim code but it always reaches this limitation.

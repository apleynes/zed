# IMPLEMENTATION_NOTES (Human inserted)

This document contains notes on the implementation of the Helix sub-module plugin for VIM inside Zed.

## Implementation Details

Attempts to re-use the existing Vim code have run into issues due to the
fundamental difference in Vim motions and Helix motions.

For example, 

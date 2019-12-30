# venn
Venn Diagram based game of deduction

This is an electronic version of a game I used to play in my sixth grade math class. The teacher would draw a Venn Diagram on the board. Above each circle would be a card. The card was face down and had a shape with a color, shape, and size.

The class was divided in two. Each team would take turns placing a face up card into one of the two circles, the overlapping portion, or outside of both. If the card matched where it was placed, it could stay. Otherwise, it was removed.

If you knew what was on the two hidden cards you could say what they were. The first team to correctly identify both hidden cards won.

# Implementation

This implementation uses the coffee crate which works pretty well for this use case.

The collection of possible choices are lined up on the left of the screen. Drag a shape into one of three areas: left circle, right circle, or the overlapping portion of both circles. If the shape has at least one property that respectively matches the left circle, the right circle, or both circles, the background of the choice will turn green. Otherwise, it will turn red.

If you place a choice in the box above the left or right circle, it must match all properties of the correct answer to turn green.

# Limitations

This version is missing a few features:
* It doesn't have an "outside the circles" capability.
* There is no win or loss detection.
* There is no turn taking.
* There is no scoring.
* There is nothing that prevents both answers from being the exact shape, size, and color.

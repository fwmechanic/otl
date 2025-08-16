\clearpage
# Outlook: The Outline Processor

Outlook is an electronic tool that helps you manipulate and organize sets of
structured notes.  You can use it to build speech outlines, organize agendas
for meetings, and construct long reports, for example.

This chapter provides complete information about all the features and
commands in Outlook.

# Activating Outlook

If you are within Outlook, you can press alt and a number key 1-9 (`a-1` - `a-9`)
to open an outline. This activates the outline specified by the number key,
bypassing the Outline Selection window and loading the default file name.

For example, `a-3` moves you to the third outline with the default file name
already loaded.  Advanced: An outliner comes up if you have built a
customized version of the SideKick Plus that specifies that `a-1` through
`a-9` bring up Outlook rather than the Notepad.

You can change the default file names with the Options menu.

You can designate your headlines as Open or Close.  When your headlines are
Open, all headlines deeper than the current one are displayed.  When you
specify Close, those headlines are hidden from view. (Use the `kp+` and `kp-`
keys to expand and contract the outline.)

Outlook can work with a pool of headlines spread across 9 windows, with a
maximum total of 2,200 headlines or 400,000 characters (whichever happens
first).

# Headline Symbols

A headline can have two symbols prefixing it:

- (triple horizontal-bar aka 'hamburger' symbol) indicates that the headline has an attached note.
- (solid right arrow) indicates deeper levels of headlines are concealed underneath that headline.

# Legend for Command Key mappings etc.

## Key naming convention
- `c-X`: Control chorded with X
- `a-X`: Alt chorded with X
- `s-X`: Shift chorded with X (X not a printable char)
- `c-X-Y`: Control chorded with X, followed by Y (prefix sequence)
- `kpX`: keypad/numpad key X (e.g., `kp+`, `kp-`, `kp*`)
- `fN`: function key N (e.g., `f2`, `f9`)

## *Command Name* name of command according to official documentation.

## Command tags:

- #coreView=should be impl in initial viewer app version
- #coreEdit=should be impl in initial editor app version

# File related

`f3` — *New Outline*
: Opens a window and allows you to type the file name of a new outline.  Press `tab` to display the default or previous file name.  If you are unsure of the name, the File Manager can help you.  Just type a drive, directory, or file name with wildcards and press enter.  If you select an existing file, Outlook loads it.  Otherwise, it creates a new outline.

`f2` or `c-k-d` — *Save Outline to File*
: Saves the current outline to its file.

`c-k-a` — *Save All Outlines*
: Saves all open outlines to their files.

`c-o-f` — *Open Outline List*
: Opens a window listing all configured outline file names.

`c-o-s` — *Save Outline List*
: Saves contents of the outline filename list window.

`a-o` — *Open File Selection*
: Opens the file selection window.

`f4` — *Print*
: Prints the marked block or whole outline (many options).

`f6` — *Switch*
: Switches to the previous open Outlook window. Otherwise, it goes to the previous application.

# Notes

Note editing commands and keystrokes are not (yet) included in this document.

# Cursor Go to and window scrolling

## Simple Cursor Movement

`c-s` or `left` — *Go to Previous Character* #coreEdit
: Move cursor left.

`c-d` or `right` — *Go to Next Character* #coreEdit
: Move cursor right.

`c-a` — *Go to Previous Word*
: Moves the cursor to the beginning of the word to the left. A word begins or ends with one of the following characters: space < > , ; ( ) [ ] A + - / $.

`c-f` — *Go to Next Word*
: Moves the cursor to the beginning of the word to the right.

`c-e` or `up` — *Go to Previous Headline* #coreView
: Moves the cursor up by one headline.

`c-x` or `down` — *Go to Next Headline* #coreView
: Moves the cursor to the headline below.

`c-w` — *Scroll Up*
: Moves the outline up by one headline. The cursor remains on the same headline until it reaches the second to the last line of the window.

`c-z` — *Scroll Down*
: Moves the outline down by one headline. The cursor remains on the same headline until it reaches the second from top line of the window.

`pgup` or `c-r` — *Go to Previous Screen* #coreView
: Moves the cursor one whole window, with an overlap of one line, nearer to the top of the outline.

`pgdn` or `c-c` — *Go to Next Screen* #coreView
: Moves the cursor one whole window, with an overlap of one line, nearer to the end of the outline.

## Extended Cursor Movement

This section describes the cursor-movement keys that perform more extensive
movements.

`c-q-s` or `home` — *Go to Start of Line* #coreEdit
: Moves the cursor to the first character of the headline.

`c-q-d` or `end` — *Go to End of Line* #coreEdit
: Moves the cursor to the position after the end of the headline.

`c-q-e` or `c-home` — *Go to Start of Window*
: Moves the cursor to the top of the window.

`c-q-x` or `c-end` — *Go to End of Window*
: Moves the cursor to the penultimate line of the window.

`c-q-r` or `c-pgup` — *Go to Start of File* #coreView
: Moves the cursor to the first headline in the outline.

`c-q-c` or `c-pgdn` — *Go to End of File* #coreView
: Moves the cursor to the last headline in the outline.

The following commands allow you to jump to special points in the outline.
(See also "Commands that Work on Several Headlines".)

`c-q-b` — *Go to Start of Block*
: Moves the cursor to the headline at the start of a marked block.

`c-q-k` — *Go to End of Block*
: Moves the cursor to the headline at the end of a marked block.

`c-q-p` — *Go to Previous Position*
: Moves to the previous position of the cursor and, if necessary, opens up the headline. This is particularly useful after loading a new outline or a Search operation.

none — *Go to Identical Level Above*
: Moves the cursor to the same level of headline above the current cursor position.

none — *Go to Identical Level Below*
: Moves the cursor to the same level of headline below the current cursor position.

`c-q-w` — *Go to Previous Attached Note*
: Moves you to the previous attached note in the outline. For Outlook to find the attached note, the headline must be open. You can also do this inside the attached note with the Previous Note command.

`c-q-z` — *Go to Next Attached Note*
: Moves you to the next attached note in the outline, whether you are in the attached note or the outline. For Outlook to find the attached note, the headline must be open. You can also do this inside the attached note with the same command.

# Insertion Commands

This section describes commands that put text into the outline.  Note that
Outlook provides a way of undoing changes to the text of a headline: Use the
*Delete Undo Headline* command described under "Deletion Commands."

`c-v` or `ins` — *Options Insert Mode* #coreEdit
: Toggles between insert and overwrite modes when entering text. When set to ON, new text is added to existing text. When set to OFF, new text replaces existing text.

`c-i` — *Insert Tab* #coreEdit
: Moves the cursor to the next tab stop in the headline. Outlook fixes the tab stops at eight-character intervals. Don't use this command frequently, as it destroys the hierarchy of the outline; however, it's handy for outline titles.

`c-m` or `enter` — *Insert Headline Current Level*
: Inserts a new headline directly below the cursor at the same level. It positions the cursor at the start of the new headline.

`alt-enter` — *Insert Headline Deeper Level*
: Inserts, below the cursor, a new headline a level deeper than the current headline and moves the cursor to it.

`kp*` or `f9` — *Insert Attached Note*
: Attaches a Notepad to the headline or, if one exists, opens the current attached note for editing. You can mark a block in the attached note and write the block to a file.

    When you press `esc`, SideKick Plus closes the attached note and returns you to Outlook, where a (triple horizontal-bar aka 'hamburger' symbol) symbol reminds you that the note exists. If you press `esc` while the note is empty, SideKick Plus ignores the attached note.

`c-q-t` — *Insert Time & Date at Cursor*
: Inserts the computer's internal time and date into the headline at the cursor position. To change the time and date format, use the Services Setup Date and Time command on the main menu.

`c-q-u` — *Insert Time & Date at End of File*
: Inserts a new headline at the end of the outline with the computer's internal time and date.

`c-q--` — *Insert Drawing Single Line*
: Allows you to draw horizontal and vertical single lines as part of the headline. Use the up, down, left, right keys to move the cursor around the outline. `esc` returns you to normal text.

`c-q-=` — *Insert Drawing Double Line*
: Allows you to draw horizontal and vertical double lines as part of the headline. `esc` returns you to normal text.

`c-q-i` — *Insert Drawing Erase Line*
: Erases lines drawn with the Insert Drawing commands. Use the up, down, left, right keys to move the cursor around the outline. `esc` returns you to normal text.

`c-k-r` — *Insert File*
: Reads a file from disk into the Outline and places it below the cursor as a marked and displayed block.

# Deletion Commands

This section describes Outlook's deletion commands.  Use the Block Delete
command to delete a large piece of text.

`c-h` or `bksp` — *Delete Previous Character*
: Deletes the character to the left of the cursor. When pressed at the start of a headline, it joins the current to the previous headline, unless the previous one is at a different level or has hidden headlines.

`c-g` or `del` — *Delete Character*
: Deletes the character above the cursor. When pressed at the end of a headline, it joins the current to the next headline, provided the next is at the same level and doesn't have hidden headlines. If it isn't, nothing happens.

`c-t` — *Delete Word*
: Deletes everything from the cursor to the end of the word. When pressed at the end of a headline, it joins the current to the next headline, provided the next is at the same level and doesn't have hidden headlines. A word is anything beginning or ending with one of the following characters: space < > , ; ( ) [ ] A + _ / $ *.

`c-q-y` — *Delete Rest of Headline*
: Deletes all text from the cursor to the end of the headline.

`c-y` — *Delete Headline*
: Deletes the current headline and any hidden headlines, if they exist. Be careful: You cannot restore the headline with the Delete Undo Headline command.

`c-q-l` — *Delete Undo Headline*
: This returns the headline to its previous form. The changes become permanent as soon as you leave the headline or use a Headline command.

# Searching and Replacing Text

`c-q-f` — *Search Find*
: Finds specified text in the outline, according to the options set by the Search Options command. If you include the wildcard '?' (or c-p-a) in the text, any character can replace the wildcard.

`c-q-a` — *Search Replace*
: Finds specified text in the outline and replaces it, according to the options set by the Search Options command. You enter the text to search for at the Search for prompt. If you include the wildcard '?' (or c-p-a), any character can replace the wildcard.

    You enter the text to substitute (or delete) at the Replace with prompt. To delete (in the outline) the Find text you've specified, don't type anything in at the Replace prompt and press enter.

    On discovering the Find text, a prompt asks whether you want to replace it. You can reply Y (for Yes) to replace it, N (for No) not to replace it, and `esc` (or `c-u`) to abort the command. Use the *Search Options Ask Before Replace* command to turn off this prompt.

   You can use any or all of the following options:

`c-q-o` — *Search Options*
: Outlook finds and replaces text in various ways. Mostly, you won't care which method is used; however, you can set your preference with the Search Options menu. To save the preferred options, use the *Options Save Setup* command.

none — *Search Options Ignore Case*
: When set to YES, ignores the difference between uppercase and lowercase.  For example, specifying "Helen" finds "Helen", "HELEN", and "helen."

none — *Search Options Global Search*
: When set to YES, replaces the text over the whole of the document regardless of where the cursor is.  When set to NO, it replaces only the first occurrence of the text after the cursor.  The Search Find command ignores the Search Options Global Search command setting.

none — *Search Options Ask Before Replace*
: When set to YES, asks you each time whether you want to replace text before doing so.  When set to NO, it does the replacement automatically.

none — *Search Options Whole Words Only*
: When set to YES, skips matching patterns embedded inside other words; for example, specifying pin will not find pineapple or supine.

none — *Search Options Sound-Alike Words*
: When set to YES, searches for words that sound like the required word; in technical terms, a Soundex search.

none — *Search Options Open Headlines*
: When set to YES, searches only the headlines revealed with the Headline Open command; otherwise, NO ignores these headlines.

none — *Search Options Marked Headlines*
: When set to YES, searches only the headlines within the marked block.

none — *Search Options Number of Times*
: Enter the number of occurrences of the string you want the search operation to work on, counted from the current cursor position.

`c-l` — *Search Again*
: Repeats the latest Search Find or Search Replace command without any prompts.

# Headline Commands

`kp+` — *Headline Open One Level*  #coreView
: Opens a level of headlines below the current headline, if possible. Each time you give this command, another level opens.

`kp-` — *Headline Close One Level* #coreView
: Closes headlines on levels below the current headline. Each time you give this command, one level closes. So you can compress the display to higher and higher levels.

`c-kp+` — *Headline Open All Levels* #coreView
: Shows all the headlines that are at a level deeper than the one the cursor is on. (Completely expand current headline/subtree.)

`c-kp-` — *Headline Close All Levels* #coreView
: Hides all the headlines that are at a level deeper than the one the cursor is on. (Completely collapse current headline/subtree.)

`c-left` or `s-tab` — *Headline Promote* #coreEdit
: Moves the headline at the cursor to a higher level, promoting it to the left.

`c-right` or `tab` — *Headline Demote* #coreEdit
: Moves the headline at the cursor to a deeper level, demoting it to the right.

`c-up` — *Headline Move Up* #coreEdit
: Exchanges the headline at the cursor with the one directly above it. This command only works within the same level of headlines.

`c-down` — *Headline Move Down* #coreEdit
: Exchanges the headline at the cursor with the one directly below it. This command only works within the same level of headlines.

`c-b` — *Headline Browse Mode*
: Toggle that affects the cursor-movement keys. When set to ON, these keys open and close levels of headlines as you move through the outline. To remind you it is on, Browse appears in the bottom left of the window border.

none — *Headline Indentation* #coreView
: Sets the number of spaces each level of headline shifts to the right. This only affects the screen appearance of the outline, not its printed or written form.

# Marking a Block of Headlines

A block of headlines is any part of the outline that you mark with some
special commands. They allow you to mark a single headline or a
continuous section of headlines. You can combine these commands to filter
out only the headlines you want.

Unlike most editors (including the Notepad) and word processors, you
mark the nearest headline, not character. The cursor can be anywhere on a
headline when you press Block Mark Begin, and the block will be marked
from the leftmost character of the headline. It also matters whether the
headline is open or closed. With an open headline, the commands act only
on that headline. With a closed headline, the commands also affect hidden
headlines.

Here are the block-marking commands:

`c-k-l` — *Block Mark Line*
: Toggles the marking of the current line. An unmarked headline becomes marked, while a marked headline becomes unmarked. If you unmark a headline in the middle of a continuous block, the middle headline becomes unmarked, splitting the block into two.

`c-k-b` or `f7` — *Block Mark Start*
: Marks the beginning of a continuous block of headlines.

`c-k-k` or `f8` — *Block Mark End*
: Marks the end of a continuous block of headlines.

`c-k-h` — *Block Mark Hide/Display*
: Switches the visual marking of the block off and on. Go to Start of Block and Go to End of Block work independently of the toggle.

# Copying, Transferring, Deleting, and Sorting a Block

Now that you know how to mark a block, let's see what you can do with it:

`c-k-c` — *Block Copy*
: Copies a previously marked block of headlines to the line after the cursor, without altering the block. When you copy a closed headline, you also copy all the headlines hidden underneath it. If the current outline contains no marked and displayed block, Outlook searches all other open outlines for marked and displayed blocks. If it finds any, it prompts you for the outline to copy from.
If you press `enter`, SideKick Plus copies the block from the lowest outline. If you type a number from 1 to 9, SideKick Plus copies the block from that outline. Once the copy is complete, the newly created block of headlines becomes the marked block.

`c-k-v` — *Block Transfer*
: Moves a previously marked block of headlines and attached notes to the headline following the cursor. When you move a closed headline, you also move everything hidden underneath it. Once it has moved, the block disappears from its original position and reappears still marked.
Use the *Headline Move Up* and *Headline Move Down* commands if you wish to move headlines within the same level.

`c-k-y` — *Block Delete*
: Deletes a previously marked block of headlines. When you delete a closed headline, you also delete everything hidden underneath it. Be careful: Once deleted, you cannot use the Delete Undo Headline command to restore the block.

`c-k-s` — *Block Sort*
: Arranges in specified order the highest level of headlines within a contiguous block, all at the same level.

    This command displays a menu that starts the sort and sets the options. Following are the four options on the menu.

   *Block Sort First Column* The place in the headline where sorting should begin.

   *Block Sort Last Column*
      The last character of the headline to be included in the sort.  Suppose
      you have the following list marked as a block with the periods
      representing blank spaces:

         Plate ..................... Part No. F12-67
         Cap ....................... Part No. F66-84
         Hub ....................... Part No. F61-90

      If you answer Block Sort First Column with 1 and Block Sort Last Column
      with 5, you get an alphabetically sorted parts list:

         Cap ....................... Part No. F66-84
         Hub ....................... Part No. F61-90
         Plate ..................... Part No. F12-67

      On the other hand, if you specify Block Sort First Column as 26 and Block
      Sort Last Column as 31, you get a numerically sorted part number list:

         Plate ..................... Part No. F12-67
         Hub ....................... Part No. F61-90
         Cap ....................... Part No. F66-84

   *Block Sort Type*
      Determines how to sort the block of headlines:
      * A ~ Z puts headlines beginning with A at the top of the marked block.
      * Z ~ A puts headlines beginning with Z at the top of the marked block.
      * RANDOM arranges headlines in a random order within the block.

# Reading Text from Other Programs

You may want to read text from another program, such as a word processor,
and convert it into an outline.  You can easily do this with the *Insert File*
command (shortcut `c-k-r`), which uses options set by the *Options Read* command.

The input file can be a text file or an outline file in Outlook, Ready,
Thinktank structured, or PC Outline structured formats.

# Options for Converting a Text File into an Outline

The *Options Read* command translates the indentations of a text file into
an outline structure.  This command affects only text files, not Outlook,
Ready, Thinktank structured, or PC Outline structured files.

`c-o-r` — *Options Read*
: Determines the conversion of the text file into an outline when using the *Insert File* command. The variations are mostly on the theme of indentation in the file, though SideKick Plus's default settings are fairly tolerant of most text files. Don't worry about changing any of the settings. They will not damage the foreign file. Use the *Options Save Setup* command to store the menu settings.

   There are three options on this menu:

none — *Options Read Minimum Indentation*
: Sets the smallest number of spaces from the left margin, before the
      current line of text creates a deeper level of headline.

none — *Options Read Tab Size*
: Sets the number of spaces a tab character (ASCII value 7) converts
      into.

none — *Options Read Graphics*
: When set to OFF, Outlook converts the text into the first 128 ASCII
      char- acters.  By doing so, it enables you to read text from, say,
      WordStar.


# Sending Outlines to Other Programs

Outlook can convert its outline into almost any text file, so you can
include it in a report or some other document.  To do this, alter the
settings of the *Options Write* and *Options Number* commands, and then use
the *Block Write to File* command to create the text file.  In this section,
you'll learn the commands connected with writing to a text file: *Block Write
to File* and *Options Write*.

`c-k-w` — *Block Write to File*
: Use this command to write part or all of an outline to a text file. You decide the appearance of the text file with the *Options Write* and *Options Number* commands. You must mark and display a block for this command to be available.

   Do not use the extensions OTL or BAK, since Outlook uses them by default.

   If the file name you enter exists, SideKick Plus asks you whether it
   should overwrite the file.  You can reply Y (for Yes) and overwrite the
   old file or N (for No) to return to the file-name prompt.

# The Options Write Command

`c-o-w` — *Options Write*
: Determines the appearance of the outline when using the *Block Write*, *Block Output Chart*, or *Block Print* commands. Use the *Options Save Setup* command to save the settings of this menu.

none — *Options Write Line Spacing*
: Sets the number of blank lines between each headline. On most printers,
      SINGLE is one-sixth of an inch, DOUBLE is one-third of an inch, and TRIPLE
      is two-thirds of an inch between each line.  Do a test run on your printer
      to verify these defaults.

none — *Options Write Indent*
: This menu changes the indentation of the outline. Following are the
      descriptions of the three menu entries.

none — *Options Write Indent Size*
: Sets the number of spaces, from the left hand margin, for each level of
      headline or attached note.  This command differs from the Headline Indent
      command, which only alters the screen appearance of the outline.  It helps
      if both these commands have the same value.

none — *Options Write Indent Character*
: Changes the type of character used to produce the indentation of the
      outline.  Use the command when you are transferring an outline to a
      foreign program.  Say you want to transfer the outline to a word
      processor but keep the indentation intact despite reformatting.  In
      this case, set the command to TAB because word processors use tab
      characters to align tables so reformat- ting ignores them.  Usually, if
      you set the command to TAB, you will also want to set the *Options Write
      Indent Size* command to 1.

none — *Options Write Indent Attached Notes*
: When set to ON, Outlook offsets the attached note to the right of the
      headline above by the number of spaces of the Options Write Indent Size
      command.  *Options Write Attached Notes* must be ON for this command to
      have any meaning.

none — *Options Write Hidden Text*
: When set to ON, writes, makes a chart of, or prints every headline
      within the block.  When set to OFF, writes, makes a chart of, or prints
      only the open headlines.

none — *Options Write Attached Notes*
: When set to ON, prints or writes all the attached notes, as well as
      each headline.  When set to OFF, only prints or writes the headlines.

none — *Options Write Structured Output*
: When set to ON, writes a text file that keeps the structure of the
      outline intact.  You can later read the file back into Outlook with the
      Insert File command, which will recreate the outline.  This is useful
      for moving or copying outlines with attached notes and sending an
      outline over electronic mail.

# Producing Numbered Headlines
When you print or write a text file, you can number the headlines with the
*Options Number* command.

`c-o-n` — *Options Number*
: Alters the format of headline numbering and the table of contents. You only see these numbers when you print or write a text file of the outline.

   The menu has two parts: a global numbering format and a local numbering
   format.  The next pages describe each option.  Together they can produce
   almost any numbering scheme imaginable.
   Use the Options Save Setup
   command to store the menu settings.

# Global Numbering of Headlines

The global numbering format choices are all in the Options Number menu.

none — *Options Number Global Type*
: Determines the style of numbering over the entire outline: OFF, PARA-
      GRAPH, or LEGAL.  When set to OFF, the headlines aren't numbered.  The
      other two choices number the headlines:

```
PARAGRAPH
I.) First Level
a.) Second Level
b.) Second Level
II.) First Level
```

```
LEGAL
1. First Level
1.1 Second Level
1.2 Second Level
2. First Level
```

none — *Options Number Minimum Width*
: Sets the number of spaces available for the number to fit into.  If
      the number is bigger than this setting, you will get an outline with a
      ragged right margin.

none — *Options Number Start level*
: Sets the level numbering should start at.

none — *Options Number End level*
: Sets the level numbering should end at.

# Local Numbering of Headlines

Each headline level can have a different type of numbering, set with the
Options Number Local menu. Following is a description of each `Options
Number Local` command.

none — *Options Number Local Level*
: Sets the numbering for each separate headline level down to level 15. You
  can specify the style of numbering you want for each level.

none — *Options Number Local Type*
: Sets the numbering style for each headline level or turns numbering OFF.
  Following are the different forms of numbering:

   | Number | Lowercase | Uppercase | Roman |
   |-------:|:---------:|:---------:|:-----:|
   | 1      | a         | A         | I     |
   | 10     | j         | J         | X     |
   | 100    | v         | V         | C     |

   There are only have 26 possible alphabetic characters, so numbering
   reverts to a or A every 26 headlines.

none — *Options Number Local Punctuation*
: Sets the character between the number and the headline. This usually
  will be either a right parenthesis or a period, but any character will do.
  Setting the Options Number Global type to LEGAL causes this setting to be
  ignored.

# Storing the Options

Following is the command that stores all the Outlook settings.

`c-o-s` — *Options Save Setup*
: Saves the settings of the Options, Sort, and Search Options menus as well as the current Outlook window size, color, and position.

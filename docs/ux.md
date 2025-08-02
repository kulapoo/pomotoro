# Pomotoro App UX Design Specification

## Overview

**Pomotoro** is a task-integrated Pomodoro timer application that combines the productivity of the Pomodoro Technique with the determination of a charging bull (toro). The app features a clean, modern interface with glass-morphism design elements, collapsible navigation, and intuitive task management.

## Layout Structure

### Sidebar Navigation (Collapsible)

- **Default Width**: 280px (expanded) / 80px (collapsed)
- **Collapsible Design**: Toggle button to hide/show sidebar content
- **App Branding**: "Pomotoro" with bull icon (🐂)
- **Three Main Sections**:
  - ⏱️ Timer (Primary view)
  - 📋 Tasks Directory
  - ⚙️ Settings

#### Sidebar States

- **Expanded State**: Full navigation with text labels
- **Collapsed State**: Icon-only navigation for space saving
- **Toggle Button**: Positioned on right edge with arrow indicators (← / →)
- **Smooth Transitions**: 0.3s ease animation between states

### Main Content Area

- **Flexible Width**: Adapts to remaining screen space
- **Centered Content**: All sections display content in center
- **Glass-morphism Background**: Translucent with backdrop blur

## Section Details

### 1. Timer Section (Default View)

#### Current Task Display

- **Task Context Box**: Shows active task information
  - Task name prominently displayed
  - Progress indicator (e.g., "Pomodoro 1 of 2 • 0 completed")
  - Subtle background highlight

#### Timer Interface

- **Large Digital Display**: 72px font size showing countdown (25:00)
- **Circular Progress Ring**: Visual progress indicator around timer
- **Session Label**: "Focus Session" above timer
- **Control Buttons**:
  - **Start** (Primary action, white background)
  - **Reset** (Secondary action)
  - **Skip Task** (Secondary action)

#### First-Time User Experience

- **Default Task**: "Getting started with Pomotoro"
- **Guided Introduction**: 2-pomodoro introductory task
- **Learning Objective**: Explore app features and Pomodoro basics

### 2. Tasks Directory Section

#### Task List Interface

Each task displays:

- **Task Header**:
  - Task title (18px, bold)
  - Status badge ("Active" or "Pending")
- **Description**: Brief task details
- **Progress Tracking**:
  - Text indicator (e.g., "0 of 3 pomodoros completed")
  - Visual progress bar with percentage fill
- **Action Button**: "Select Task" or "Currently Active"

#### Task States

- **Active Task**:
  - Teal accent color (#4ecdc4)
  - Enhanced visual prominence
  - "Currently Active" button (non-clickable)
  - Glowing shadow effect

- **Pending Tasks**:
  - Standard gray/white styling
  - Red accent border (#ff6b6b)
  - Clickable "Select Task" button
  - Hover effects for interactivity

#### Default Tasks for New Users

1. **"Getting started with Pomotoro"** (Active by default)
   - 2 pomodoros estimated
   - Introduces app functionality
2. **"Complete project wireframes"** (Example task)
   - 3 pomodoros estimated
3. **"Review code documentation"** (Partially completed example)
   - 4 pomodoros total, 1 completed (25% progress)

#### Task Management Actions

- **Add New Task**: Primary button at bottom
- **Task Selection**: One-click switching between tasks
- **Auto-Navigation**: Selecting task automatically switches to Timer view

### 3. Settings Section

#### Configuration Options

- **Focus Duration**:
  - Number input (1-60 minutes)
  - Default: 25 minutes
  - Helper text: "Minutes per focus session"

- **Short Break**:
  - Number input (1-30 minutes)
  - Default: 5 minutes
  - Helper text: "Minutes for short break"

- **Long Break**:
  - Number input (1-60 minutes)
  - Default: 15 minutes
  - Helper text: "Minutes for long break (every 4 sessions)"

- **Notifications**:
  - Checkbox: Sound notifications (enabled by default)
  - Checkbox: Desktop notifications (enabled by default)

## User Interactions & Flow

### Sidebar Navigation

- **Toggle Functionality**: Click arrow button to collapse/expand sidebar
- **Space Optimization**: Collapsed mode shows only icons for minimal footprint
- **Active State Highlighting**: Current section highlighted in both expanded and collapsed states
- **Smooth Transitions**: Fade effects for text labels and width changes

### Navigation Behavior

- **Active State Highlighting**: Current section highlighted in sidebar
- **Smooth Transitions**: Fade effects between sections
- **Hover Effects**: Interactive feedback on all clickable elements

### Task-Timer Integration

1. **Task Selection Flow**:
   - User clicks "Select Task" in Tasks Directory
   - Task becomes active (visual state change)
   - User automatically navigated to Timer view
   - Timer displays selected task context

2. **Progress Tracking**:
   - Completed pomodoros update task progress
   - Progress bars reflect completion percentage
   - Task status updates in real-time

3. **Default Experience**:
   - New users start with introductory task
   - Clear guidance for first session
   - Seamless transition to personal tasks

### Responsive Behaviors

- **Button States**: Hover effects with transform and shadow
- **Task Highlighting**: Active task has distinct visual treatment
- **Feedback Systems**: Clear visual confirmation of actions

## Visual Design System

### Color Palette

- **Primary Gradient**: Purple to blue (#667eea to #764ba2)
- **Active Accent**: Teal (#4ecdc4)
- **Task Accent**: Red-orange (#ff6b6b)
- **Text**: White for main content, dark gray for sidebar
- **Backgrounds**: Translucent white with backdrop blur

### Typography

- **Font Family**: Segoe UI system font stack
- **Timer Display**: 72px, lightweight
- **Section Titles**: 32px
- **Task Titles**: 18px, semi-bold
- **Body Text**: 16px standard weight

### Spacing & Layout

- **Sidebar**: 280px fixed width
- **Content Padding**: 40px all sides
- **Component Spacing**: 15-30px between elements
- **Border Radius**: 12px for cards, 25px for buttons

### Visual Effects

- **Glass-morphism**: Translucent panels with backdrop blur
- **Shadows**: Subtle depth for elevation
- **Gradients**: Modern background treatments
- **Transitions**: 0.3s ease for all interactive elements

## Accessibility Considerations

### Visual Accessibility

- **High Contrast**: White text on gradient backgrounds
- **Clear Focus States**: Visible button and navigation states
- **Readable Typography**: Large, clear fonts throughout
- **Color + Text**: Status communicated via both color and text labels

### Interaction Accessibility

- **Keyboard Navigation**: All interactive elements accessible
- **Clear Labels**: Descriptive button text and form labels
- **Feedback**: Visual confirmation of all user actions
- **Consistent Patterns**: Predictable interaction behaviors

## Technical Implementation Notes

### State Management

- **Active Task Tracking**: Single task can be active at a time
- **Progress Persistence**: Task progress maintained across sessions
- **Settings Storage**: User preferences retained between uses
- **Sidebar State**: Toggle position remembered across sessions

### Navigation Logic

- **Section Switching**: Hide/show content areas based on navigation
- **Auto-Navigation**: Task selection triggers view change
- **Active State Management**: Visual indicators for current section/task
- **Sidebar Toggle**: CSS class-based state management with smooth transitions

### Timer Functionality

- **Countdown Logic**: 25-minute default with customizable duration
- **Session Tracking**: Progress updates tied to task completion
- **Break Integration**: Automatic transitions between work and break periods

This design creates an intuitive, focused experience that seamlessly integrates task management with time tracking, helping users maintain clarity about their work objectives while staying productive with the Pomodoro Technique.

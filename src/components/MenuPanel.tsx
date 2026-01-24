import React from 'react'
import { Menubar } from 'radix-ui'

const MenuPanel = () => {
  return (
    <Menubar.Root className='MenubarRoot'>
      <Menubar.Menu>
        <Menubar.Trigger className='MenubarTrigger'>File</Menubar.Trigger>
        <Menubar.Portal>
          <Menubar.Content
            className='MenubarContent'
            align='start'
            sideOffset={5}
            alignOffset={-3}
          >
            <Menubar.Item className='MenubarItem'>New</Menubar.Item>
            <Menubar.Item className='MenubarItem'>
              New from template
            </Menubar.Item>
            <Menubar.Separator className='MenubarSeparator' />
            <Menubar.Item className='MenubarItem'>
              Save<div className='RightSlot'>Ctrl + S</div>
            </Menubar.Item>
            <Menubar.Item className='MenubarItem'>
              Save as...<div className='RightSlot'>Shift + Ctrl + S</div>
            </Menubar.Item>
            <Menubar.Separator className='MenubarSeparator' />
            <Menubar.Item className='MenubarItem'>Export</Menubar.Item>
          </Menubar.Content>
        </Menubar.Portal>
      </Menubar.Menu>
      <Menubar.Menu>
        <Menubar.Trigger className='MenubarTrigger'>Options</Menubar.Trigger>
        <Menubar.Portal>
          <Menubar.Content
            className='MenubarContent'
            align='start'
            sideOffset={5}
            alignOffset={-3}
          >
            <Menubar.Item className='MenubarItem'>
              General settings
            </Menubar.Item>
            <Menubar.Item className='MenubarItem'>Audio settings</Menubar.Item>
            <Menubar.Item className='MenubarItem'>MIDI settings</Menubar.Item>
            <Menubar.Item className='MenubarItem'>File settings</Menubar.Item>
            <Menubar.Item className='MenubarItem'>Manage plugins</Menubar.Item>
            <Menubar.Separator className='MenubarSeparator' />
            <Menubar.Item className='MenubarItem'>
              Project settings
            </Menubar.Item>
            <Menubar.Item className='MenubarItem'>Project info</Menubar.Item>
            <Menubar.Separator className='MenubarSeparator' />
            <Menubar.Item className='MenubarItem'>Debugging log</Menubar.Item>
          </Menubar.Content>
        </Menubar.Portal>
      </Menubar.Menu>
    </Menubar.Root>
  )
}

export default MenuPanel

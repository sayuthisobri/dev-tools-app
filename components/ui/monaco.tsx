'use client'
import React, {useEffect} from 'react'
import {Editor, EditorProps, Monaco as _monaco, useMonaco} from '@monaco-editor/react'

// loader.config({monaco: _monaco})

export default function Monaco({height, defaultLanguage, theme, ...props}: EditorProps): React.ReactNode {
  const monaco = useMonaco()
  useEffect(() => {
    if (!monaco) return

    // monaco.editor.create(document.getElementById('container'), {
    //   value: 'function hello() {\n\tconsole.log("Hello, world!");\n}',
    //   language: 'javascript',
    // })

  }, [monaco])

  function onMount(editor: any, monaco: _monaco) {
    console.log('onMount', editor, monaco)
    monaco.editor.defineTheme('vs-dark', {
      base: 'vs-dark', // can also be vs-dark or hc-black
      inherit: true,
      rules: [],
      colors: {
        'editor.background': '#00000020',
      },
    })
    monaco.editor.setTheme('vs-dark')
  }

  return (
    <>
      <Editor height={height} defaultLanguage={defaultLanguage ?? 'javascript'}
              theme={theme ?? 'vs-dark'} {...props} onMount={onMount}
              options={{
                automaticLayout: true,
                minimap: {
                  enabled: false, // Disable the minimap
                },
              }}

      ></Editor>
    </>
  )
}

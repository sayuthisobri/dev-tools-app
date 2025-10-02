'use client'
import React, { useEffect, useRef, useState } from 'react'
import { Editor, EditorProps, Monaco as _monaco } from '@monaco-editor/react'
import { editor as editorNS } from 'monaco-editor'
import { debounce } from 'lodash'

// loader.config({monaco: _monaco})

export default function Monaco({height, defaultLanguage, theme, ...props}: EditorProps): React.ReactNode {
  const editorRef = useRef<HTMLDivElement>(null)
  const [editor, setEditor] = useState<editorNS.IStandaloneCodeEditor | null>(null)
  useEffect(() => {
    if (!editor || !editorRef.current) return

    const resetEditorLayout = () => {
      editor.layout({width: 0, height: 0})

      window.requestAnimationFrame(() => {
        const rect = editorRef.current?.getBoundingClientRect()
        // console.log('resize to', rect)
        if (!rect) return
        editor.layout({width: rect.width, height: rect.height})
      })
    }
    const debounced = debounce(resetEditorLayout, 300)
    window.addEventListener('resize', debounced)
    return () => window.removeEventListener('resize', debounced)

  }, [editorRef, editor])

  function onMount(editor: editorNS.IStandaloneCodeEditor, monaco: _monaco) {
    console.log('onMount', editor, monaco)
    setEditor(editor)
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
    <div ref={editorRef} className="h-full">
      <Editor defaultLanguage={defaultLanguage ?? 'javascript'}
              theme={theme ?? 'vs-dark'} {...props} onMount={onMount}
              options={{
                // automaticLayout: true,
                minimap: {
                  enabled: false, // Disable the minimap
                },
              }}

      ></Editor>
    </div>
  )
}

"use client"

import { ChevronRightIcon, FileIcon, FolderIcon, FolderOpenIcon } from "lucide-react"
import type { KeyboardEvent } from "react"
import { createContext, type HTMLAttributes, type ReactNode, useContext, useState } from "react"
import { Collapsible, CollapsibleContent, CollapsibleTrigger } from "@/components/ui/collapsible"
import { cn } from "@/lib/utils"

interface FileTreeContextType {
  expandedPaths: Set<string>
  togglePath: (path: string) => void
  selectedPath?: string
  onSelect?: (path: string) => void
}

const FileTreeContext = createContext<FileTreeContextType>({
  expandedPaths: new Set(),
  togglePath: () => undefined,
})

export type FileTreeProps = HTMLAttributes<HTMLDivElement> & {
  expanded?: Set<string>
  defaultExpanded?: Set<string>
  selectedPath?: string
  onSelect?: (path: string) => void
  onExpandedChange?: (expanded: Set<string>) => void
}

export const FileTree = ({
  expanded: controlledExpanded,
  defaultExpanded = new Set(),
  selectedPath,
  onSelect,
  onExpandedChange,
  className,
  children,
  ...props
}: FileTreeProps) => {
  const [internalExpanded, setInternalExpanded] = useState(defaultExpanded)
  const isControlled = controlledExpanded !== undefined
  const expandedPaths = isControlled ? controlledExpanded : internalExpanded

  const togglePath = (path: string) => {
    const newExpanded = new Set(expandedPaths)
    if (newExpanded.has(path)) {
      newExpanded.delete(path)
    } else {
      newExpanded.add(path)
    }
    if (!isControlled) {
      setInternalExpanded(newExpanded)
    }
    onExpandedChange?.(newExpanded)
  }

  return (
    <FileTreeContext.Provider value={{ expandedPaths, togglePath, selectedPath, onSelect }}>
      <div
        className={cn("rounded-lg border bg-aurora-bg font-mono text-sm", className)}
        role="tree"
        {...props}
      >
        <div className="p-2">{children}</div>
      </div>
    </FileTreeContext.Provider>
  )
}

export type FileTreeFolderProps = HTMLAttributes<HTMLDivElement> & {
  path: string
  name: string
}

export const FileTreeFolder = ({
  path,
  name,
  className,
  children,
  ...props
}: FileTreeFolderProps) => {
  const { expandedPaths, togglePath, selectedPath, onSelect } = useContext(FileTreeContext)
  const isExpanded = expandedPaths.has(path)
  const isSelected = selectedPath === path

  return (
    <Collapsible onOpenChange={() => togglePath(path)} open={isExpanded}>
      {/* role="treeitem" on the wrapper but not tabIndex — the inner
          <button> is already focusable, so adding tabIndex here creates a
          duplicate tab stop that lands on a div with no handlers. */}
      <div className={cn("", className)} role="treeitem" {...props}>
        <CollapsibleTrigger asChild>
          <button
            className={cn(
              "flex w-full items-center gap-1 rounded px-2 py-1 text-left transition-colors hover:bg-aurora-panel-muted/50",
              isSelected && "bg-aurora-panel-muted",
            )}
            onClick={() => onSelect?.(path)}
            type="button"
          >
            <ChevronRightIcon
              className={cn(
                "size-4 shrink-0 text-aurora-text-muted transition-transform",
                isExpanded && "rotate-90",
              )}
            />
            <FileTreeIcon>
              {isExpanded ? (
                <FolderOpenIcon className="size-4 text-blue-500" />
              ) : (
                <FolderIcon className="size-4 text-blue-500" />
              )}
            </FileTreeIcon>
            <FileTreeName>{name}</FileTreeName>
          </button>
        </CollapsibleTrigger>
        <CollapsibleContent>
          <div className="ml-4 border-l pl-2">{children}</div>
        </CollapsibleContent>
      </div>
    </Collapsible>
  )
}

export type FileTreeFileProps = HTMLAttributes<HTMLDivElement> & {
  path: string
  name: string
  icon?: ReactNode
}

export const FileTreeFile = ({
  path,
  name,
  icon,
  className,
  children,
  ...props
}: FileTreeFileProps) => {
  const { selectedPath, onSelect } = useContext(FileTreeContext)
  const isSelected = selectedPath === path

  const handleKeyDown = (e: KeyboardEvent<HTMLDivElement>) => {
    if (e.key === "Enter" || e.key === " ") {
      // Space would otherwise scroll the page on a focusable div.
      e.preventDefault()
      onSelect?.(path)
    }
  }

  return (
    <div
      className={cn(
        "flex cursor-pointer items-center gap-1 rounded px-2 py-1 transition-colors hover:bg-aurora-panel-muted/50",
        isSelected && "bg-aurora-panel-muted",
        className,
      )}
      role="treeitem"
      tabIndex={0}
      {...props}
      onClick={() => onSelect?.(path)}
      onKeyDown={handleKeyDown}
    >
      {children ?? (
        <>
          <span className="size-4" />
          <FileTreeIcon>
            {icon ?? <FileIcon className="size-4 text-aurora-text-muted" />}
          </FileTreeIcon>
          <FileTreeName>{name}</FileTreeName>
        </>
      )}
    </div>
  )
}

export type FileTreeIconProps = HTMLAttributes<HTMLSpanElement>

export const FileTreeIcon = ({ className, children, ...props }: FileTreeIconProps) => (
  <span className={cn("shrink-0", className)} {...props}>
    {children}
  </span>
)

export type FileTreeNameProps = HTMLAttributes<HTMLSpanElement>

export const FileTreeName = ({ className, children, ...props }: FileTreeNameProps) => (
  <span className={cn("truncate", className)} {...props}>
    {children}
  </span>
)

export type FileTreeActionsProps = HTMLAttributes<HTMLDivElement>

export const FileTreeActions = ({ className, children, ...props }: FileTreeActionsProps) => (
  // Spread props first so the internal stopPropagation handlers can't be
  // silently overridden by a consumer's onClick/onKeyDown.
  <div
    className={cn("ml-auto flex items-center gap-1", className)}
    role="group"
    {...props}
    onClick={e => {
      props.onClick?.(e)
      e.stopPropagation()
    }}
    onKeyDown={e => {
      props.onKeyDown?.(e)
      e.stopPropagation()
    }}
  >
    {children}
  </div>
)

/** Demo component for preview */
export default function FileTreeDemo() {
  const [selected, setSelected] = useState<string | undefined>()

  return (
    <div className="w-full max-w-xs p-4">
      <FileTree
        defaultExpanded={new Set(["src", "src/components"])}
        selectedPath={selected}
        onSelect={path => setSelected(path)}
      >
        <FileTreeFolder path="src" name="src">
          <FileTreeFolder path="src/components" name="components">
            <FileTreeFile path="src/components/Button.tsx" name="Button.tsx" />
            <FileTreeFile path="src/components/Card.tsx" name="Card.tsx" />
          </FileTreeFolder>
          <FileTreeFile path="src/index.ts" name="index.ts" />
        </FileTreeFolder>
        <FileTreeFile path="package.json" name="package.json" />
        <FileTreeFile path="README.md" name="README.md" />
      </FileTree>
    </div>
  )
}

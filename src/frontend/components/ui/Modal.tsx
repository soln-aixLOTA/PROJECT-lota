import { cn } from '@/lib/utils';
import { XMarkIcon } from '@heroicons/react/24/outline';
import * as Dialog from '@radix-ui/react-dialog';

const Modal = Dialog.Root;
const ModalTrigger = Dialog.Trigger;

const ModalContent = ({
  className,
  children,
  ...props
}: Dialog.DialogContentProps) => (
  <Dialog.Portal>
    <Dialog.Overlay className="fixed inset-0 bg-black/50 backdrop-blur-sm" />
    <Dialog.Content
      className={cn(
        "fixed left-[50%] top-[50%] translate-x-[-50%] translate-y-[-50%] w-full max-w-lg bg-background rounded-lg p-6 shadow-lg",
        className
      )}
      {...props}
    >
      {children}
      <Dialog.Close className="absolute right-4 top-4 rounded-sm opacity-70 ring-offset-background transition-opacity hover:opacity-100">
        <XMarkIcon className="h-4 w-4" />
        <span className="sr-only">Close</span>
      </Dialog.Close>
    </Dialog.Content>
  </Dialog.Portal>
);

const ModalHeader = ({
  className,
  ...props
}: React.HTMLAttributes<HTMLDivElement>) => (
  <div
    className={cn("flex flex-col space-y-1.5 text-center sm:text-left", className)}
    {...props}
  />
);

const ModalFooter = ({
  className,
  ...props
}: React.HTMLAttributes<HTMLDivElement>) => (
  <div
    className={cn("flex flex-col-reverse sm:flex-row sm:justify-end sm:space-x-2", className)}
    {...props}
  />
);

const ModalTitle = Dialog.Title;
const ModalDescription = Dialog.Description;

export {
    Modal, ModalContent, ModalDescription, ModalFooter, ModalHeader, ModalTitle, ModalTrigger
};


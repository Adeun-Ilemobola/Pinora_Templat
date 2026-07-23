
import { useNavigate } from "react-router-dom";


import { Button } from '@/components/ui/button'

export default function NotFound() {
    const navigate = useNavigate();
  return (
    <div className=' min-h-screen flex flex-col  justify-center items-center gap-4'>

        <h1 className=" text-5xl">NotFound</h1>
        <Button
        className=" text-3xl"
        variant={"destructive"}
        onClick={()=>{
            navigate("/")
        }}
        >
            Back
        </Button>


      
    </div>
  )
}
